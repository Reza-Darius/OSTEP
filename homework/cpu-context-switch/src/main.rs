use std::{
    env,
    error::Error,
    io::{PipeReader, PipeWriter, Read, Write},
    time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        eprintln!("please provide the number of iterations");
        std::process::exit(1)
    }

    let (p1_reader, p1_writer) = std::io::pipe()?;
    let (p2_reader, p2_writer) = std::io::pipe()?;

    let n_measurements: u32 = args[0].parse::<u32>()? / 2;
    let thread_measures =
        std::thread::spawn(move || ping_pong(p1_reader, p2_writer, n_measurements));

    let main_measure = ping_pong(p2_reader, p1_writer, n_measurements);
    let total = thread_measures.join().unwrap() + main_measure;

    println!(
        "average duration {:?}",
        total.div_f32(n_measurements as f32)
    );
    Ok(())
}

fn ping_pong(mut reader: PipeReader, mut writer: PipeWriter, n_measures: u32) -> Duration {
    let mut buf = [0; 1];
    let mut total: Duration = Duration::new(0, 0);

    for _ in 0..n_measures {
        let now = quanta::Instant::now();
        let _ = writer.write(&buf);
        total += now.elapsed();

        let now = quanta::Instant::now();
        let _ = reader.read(&mut buf);
        total += now.elapsed();
    }
    total
}
