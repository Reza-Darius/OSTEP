use std::{
    env,
    error::Error,
    io::{Read, Write},
    time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        eprintln!("please provide the number of iterations");
        std::process::exit(1)
    }

    let (mut p1_reader, mut p1_writer) = std::io::pipe()?;
    let (mut p2_reader, mut p2_writer) = std::io::pipe()?;

    let _ = std::thread::spawn(move || {
        let mut buf = [0; 1];
        loop {
            let _ = p2_writer.write(&buf).unwrap();
            let _ = p1_reader.read(&mut buf).unwrap();
        }
    });

    let n_measurements: u32 = args[0].parse::<u32>()?;
    let mut total: Duration = Duration::new(0, 0);
    let mut buf = [0; 1];

    for _ in 0..n_measurements {
        let now = quanta::Instant::now();
        let _ = p2_reader.read(&mut buf).unwrap();
        total += now.elapsed();

        let now = quanta::Instant::now();
        let _ = p1_writer.write(&buf).unwrap();
        total += now.elapsed();
    }

    println!(
        "average duration {:?}",
        total.div_f32(n_measurements as f32)
    );
    Ok(())
}
