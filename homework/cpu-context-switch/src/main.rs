use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    mem,
    path::Path,
    process::Stdio,
};
use tokio::io::AsyncWriteExt;
use tokio::{io::AsyncReadExt, task::JoinSet};

const MSG_LEN: usize = 1;
const CHILD_PATH: &str = "target/release/child_process";

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let num_iterations: u32 = args[0].parse::<u32>()?;

    if args.is_empty() {
        return Err("please provide the number of iterations".into());
    }

    // we pin the process to a CPU core to reduce any measurement noise caused by cache
    // invalidiation
    unsafe {
        let mut set: libc::cpu_set_t = mem::zeroed();
        libc::CPU_SET(0, &mut set);
        let pid = libc::getpid();
        if libc::sched_setaffinity(pid, mem::size_of::<libc::cpu_set_t>(), &mut set as *const _)
            == -1
        {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to set cpu affinity {}", err);
            return Err(err.into());
        }
    }
    println!("testing processes:");
    measure_cw(num_iterations, process_test)?;
    println!();

    println!("testing os threads:");
    measure_cw(num_iterations, thread_test)?;
    println!();

    println!("testing green threads with channel:");
    measure_cw(num_iterations, async_pipes)?;
    println!();

    println!("testing green threads with channel:");
    measure_cw(num_iterations, async_channel)?;
    Ok(())
}

fn process_test(num_iterations: u32) -> Result<(), Box<dyn Error>> {
    let path = Path::new(CHILD_PATH);

    let mut child = std::process::Command::new(path)
        .arg(num_iterations.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut writer = child.stdin.take().unwrap();
    let reader = child.stdout.take().unwrap();

    // write one thing such that the thread doesnt block indefinitely
    let _ = writer.write(&[0u8; 1]).unwrap();

    let now = quanta::Instant::now();
    ping_pong(reader, writer, num_iterations);
    let elapsed = now.elapsed();
    let n_switches = (num_iterations * 2) as f32;
    println!(
        "average context switch duration {:?}",
        elapsed.div_f32(n_switches)
    );
    child.kill()?;
    Ok(())
}

fn thread_test(num_iterations: u32) -> Result<(), Box<dyn Error>> {
    let (p1_reader, mut p1_writer) = io::pipe()?;
    let (p2_reader, p2_writer) = io::pipe()?;

    _ = std::thread::spawn(move || {
        ping_pong(p1_reader, p2_writer, num_iterations);
    });

    // write one thing such that the thread doesnt block indefinitely
    let _ = p1_writer.write(&[0u8; 1]).unwrap();

    let now = quanta::Instant::now();
    ping_pong(p2_reader, p1_writer, num_iterations);
    let elapsed = now.elapsed();
    let n_switches = (num_iterations * 2) as f32;
    println!(
        "average context switch duration {:?}",
        elapsed.div_f32(n_switches)
    );
    Ok(())
}

fn async_channel(num_iterations: u32) -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let (p1_sender, mut p1_receiver) = tokio::sync::mpsc::channel::<u8>(1);
        let (p2_sender, mut p2_receiver) = tokio::sync::mpsc::channel::<u8>(1);
        let mut set = JoinSet::new();

        set.spawn(async move {
            for _ in 0..num_iterations {
                let _ = p1_receiver.recv().await;
                let _ = p2_sender.send(42).await;
            }
        });

        set.spawn(async move {
            let now = quanta::Instant::now();
            for _ in 0..num_iterations {
                let _ = p1_sender.send(67).await;
                let _ = p2_receiver.recv().await;
            }

            let elapsed = now.elapsed();
            let n_switches = (num_iterations * 2) as f32;
            println!(
                "average context switch duration {:?}",
                elapsed.div_f32(n_switches)
            );
        });

        set.join_all().await;
        Ok::<_, Box<dyn Error>>(())
    })
}

fn async_pipes(num_iterations: u32) -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let (mut p1_sender, mut p1_receiver) = tokio::net::unix::pipe::pipe()?;
        let (mut p2_sender, mut p2_receiver) = tokio::net::unix::pipe::pipe()?;
        let mut set = JoinSet::new();

        set.spawn(async move {
            let mut buf = [0; MSG_LEN];
            for _ in 0..num_iterations {
                let _ = p1_receiver.read(&mut buf).await;
                let _ = p2_sender.write(&buf).await;
            }
        });

        set.spawn(async move {

            let now = quanta::Instant::now();
            let mut buf = [0; MSG_LEN];
            for _ in 0..num_iterations {
                let _ = p1_sender.write(&buf).await;
                let _ = p2_receiver.read(&mut buf).await;
            }

            let elapsed = now.elapsed();
            let n_switches = (num_iterations * 2) as f32;
            println!(
                "average context switch duration {:?}",
                elapsed.div_f32(n_switches)
            );
        });
        set.join_all().await;
        Ok::<_, Box<dyn Error>>(())
    })
}

fn measure_cw(
    num_iterations: u32,
    test: impl FnOnce(u32) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let (pre_vol_cw, pre_invol_cw) = context_switch_info()?;
    test(num_iterations)?;
    let (post_vol_cw, post_invol_cw) = context_switch_info()?;
    println!(
        "number of voluntary switches: {}\n\
        number of involunatry switches {}",
        post_vol_cw - pre_vol_cw,
        post_invol_cw - pre_invol_cw
    );
    Ok(())
}

fn context_switch_info() -> Result<(i64, i64), Box<dyn Error>> {
    unsafe {
        let mut usage: libc::rusage = mem::zeroed();
        if libc::getrusage(libc::RUSAGE_THREAD, &mut usage as *mut _) == -1 {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to call getrusage() {}", err);
            return Err(err.into());
        }
        Ok((usage.ru_nvcsw, usage.ru_nivcsw))
    }
}

fn ping_pong(mut reader: impl Read, mut writer: impl Write, num_iterations: u32) {
    let mut buf = [0; MSG_LEN];
    for _ in 0..num_iterations {
        let _ = reader.read(&mut buf);
        let _ = writer.write(&buf);
    }
}
