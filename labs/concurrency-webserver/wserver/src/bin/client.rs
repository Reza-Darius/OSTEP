//! setup sending http request
//!
//! arguments:
//! -number of requests to send per thread
//! -number of threads
//!
//! randomize which file to request
//! measure turnaround time for each file type
//! mease turnaround time for every file

use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;

const SMALL: &str = "/files/file1.txt";
const MEDIUM: &str = "/files/file2.txt";
const LARGE: &str = "/files/file3.txt";

const FILES: [&str; 3] = [SMALL, MEDIUM, LARGE];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'm')]
    n_messages: u32,
    #[arg(short)]
    port: u16,
    #[arg(short)]
    threads: u16,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let f = send_msg("/files/file1.txt", 8000)?;
    println!("file1 one took: {:?}", f);

    let f = send_msg("/files/file2.txt", 8000)?;
    println!("file2 one took: {:?}", f);

    let f = send_msg("/files/file3.txt", 8000)?;
    println!("file3 one took: {:?}", f);

    let worker_count = args.n_messages / args.threads as u32;

    let res = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..args.threads {
            handles.push(s.spawn(|| worker(worker_count, args.port)));
        }

        let mut res = Vec::new();
        for handle in handles {
            res.push(handle.join().unwrap());
        }
        res
    });

    aggregate_results(res);
    Ok(())
}

fn aggregate_results(data: Vec<HashMap<&'static str, (Duration, u32)>>) {
    let res: HashMap<&'static str, (Duration, u32)> =
        data.iter().fold(HashMap::new(), |mut acc, map| {
            for file in FILES {
                if let Some(entry) = map.get(file).copied() {
                    acc.entry(file)
                        .and_modify(|(durr, count)| {
                            *durr += entry.0;
                            *count += entry.1;
                        })
                        .or_insert(entry);
                };
            }
            acc
        });

    for (file, (durr, count)) in res {
        let average = durr / count;
        println!("average time for {}: {:?}", file, average);
    }
}

fn worker(count: u32, port: u16) -> HashMap<&'static str, (Duration, u32)> {
    let mut map = HashMap::new();
    for _ in 0..count {
        let idx = rand::random_range(0..3);
        let r = send_msg(FILES[idx], port).unwrap();

        map.entry(FILES[idx])
            .and_modify(|(dur, count)| {
                *dur += r;
                *count += 1;
            })
            .or_insert((r, 1));
    }
    map
}

fn send_msg(file: &str, port: u16) -> Result<Duration> {
    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))?;
    write!(
        stream,
        "GET {} HTTP/1.0\r\n\
        \r\n",
        file
    )?;
    let mut buf = Vec::new();
    let now = Instant::now();
    let _ = stream.read_to_end(&mut buf)?;

    assert!(buf.starts_with("HTTP/1.0 200 OK\r\n".as_bytes()));
    Ok(now.elapsed())
}
