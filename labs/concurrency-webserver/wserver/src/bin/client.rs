// setup sending http request
//
// arguments:
// -number of requests to send per thread
// -number of threads
//
// randomize which file to request
// measure turnaround time for each file type
// mease turnaround time for every file

use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
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
    let addr = format!("127.0.0.1:{}", args.port);

    let res = std::thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..args.threads {
            handles.push(s.spawn(|| worker(args.n_messages, &addr)));
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

    let res = res
        .into_iter()
        .map(|(file, (durr, count))| (file, durr / count))
        .collect::<BTreeMap<_, _>>();

    let n_files = res.len();
    let mut total_time = Duration::default();

    for (file, average) in res {
        println!("average turnaround time for {}: {:?}", file, average);
        total_time += average;
    }

    println!("total average: {:?}", total_time / n_files as u32);
}

fn worker(n_msgs: u32, addr: impl ToSocketAddrs) -> HashMap<&'static str, (Duration, u32)> {
    let mut map = HashMap::new();
    let mut buf = Vec::new();

    for _ in 0..n_msgs {
        let idx = rand::random_range(0..3);
        let measurement = send_msg(FILES[idx], &mut buf, &addr).unwrap();

        map.entry(FILES[idx])
            .and_modify(|(dur, count)| {
                *dur += measurement;
                *count += 1;
            })
            .or_insert((measurement, 1));

        buf.clear();
    }
    map
}

fn send_msg(file: &str, buf: &mut Vec<u8>, addr: impl ToSocketAddrs) -> Result<Duration> {
    let mut stream = std::net::TcpStream::connect(addr)?;
    write!(
        stream,
        "GET {} HTTP/1.0\r\n\
        \r\n",
        file
    )?;

    // measure the response
    let now = Instant::now();
    stream.read_to_end(buf)?;
    let elapsed = now.elapsed();

    assert!(buf.starts_with("HTTP/1.0 200 OK\r\n".as_bytes()));

    Ok(elapsed)
}
