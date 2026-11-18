use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    mem,
    path::Path,
    process::Stdio,
};

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
    process_test(num_iterations, Path::new(CHILD_PATH))?;

    println!("testing threads:");
    thread_test(num_iterations)?;
    Ok(())
}

fn process_test(num_iterations: u32, child_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut child = std::process::Command::new(child_path)
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
    context_switch_info()?;
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
    context_switch_info()?;
    Ok(())
}

fn context_switch_info() -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut usage: libc::rusage = mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut usage as *mut _) == -1 {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to call getrusage() {}", err);
            return Err(err.into());
        }

        println!(
            "number of voluntary switches: {}\n\
            number of involunatry switches {}",
            usage.ru_nvcsw, usage.ru_nivcsw
        )
    }
    Ok(())
}

fn ping_pong(mut reader: impl Read, mut writer: impl Write, num_iterations: u32) {
    let mut buf = [0; MSG_LEN];
    for _ in 0..num_iterations {
        let _ = reader.read(&mut buf);
        let _ = writer.write(&buf);
    }
}
