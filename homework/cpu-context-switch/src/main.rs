use std::{
    env,
    error::Error,
    io::{PipeReader, PipeWriter, Read, Write},
    time::Duration,
};

use rustix::thread::CpuSet;

const MSG_LEN: usize = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let num_iterations: u32 = args[0].parse::<u32>()?;

    // we pin the process to a CPU core to reduce any measurement noise caused by cache
    // invalidiation
    let mut set = CpuSet::new();
    set.set(0);
    rustix::thread::sched_setaffinity(None, &set)?;

    if args.is_empty() {
        eprintln!("please provide the number of iterations");
        std::process::exit(1)
    }

    let (p1_reader, mut p1_writer) = std::io::pipe()?;
    let (p2_reader, p2_writer) = std::io::pipe()?;

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

fn ping_pong(mut reader: PipeReader, mut writer: PipeWriter, num_iterations: u32) {
    let mut buf = [0; MSG_LEN];
    for _ in 0..num_iterations {
        let _ = reader.read(&mut buf);
        let _ = writer.write(&buf);
    }
}
