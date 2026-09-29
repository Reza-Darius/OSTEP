/*
* HTTP file server over a threadpool
* 1. parse http
* 2. setup thread pool
* 3. setup server
*/

use anyhow::Result;
use clap::Parser;
use std::{net::TcpListener, os::unix::fs::MetadataExt, path::PathBuf};
use wserver::{
    chan::Channel,
    handler::{Job, Policy, worker},
    http::read_stream,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'd', default_value = ".")]
    basedir: PathBuf,
    #[arg(short)]
    port: u32,
    #[arg(short, default_value_t = 1)]
    threads: usize,
    #[arg(short, default_value_t = 1)]
    buffer: usize,
    #[arg(short, default_value_t)]
    schedule: Policy,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let w_count = worker_count(args.threads);
    let queue = Channel::new(args.buffer);
    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&addr)?;

    for _ in 0..w_count {
        let q = queue.clone();
        std::thread::spawn(move || worker(q));
    }

    eprintln!(
        "listening on {addr}, with {w_count} threads, buffer {}, schedule {}",
        args.buffer, args.schedule
    );

    match args.schedule {
        Policy::Fifo => fifo_server(listener, queue)?,
        Policy::Sff => todo!(),
    }

    Ok(())
}

fn fifo_server(listener: TcpListener, queue: Channel) -> Result<()> {
    while let Ok((stream, _)) = listener.accept() {
        queue.push_back(Job {
            client: stream,
            path: None,
            file_size: None,
        });
    }
    Ok(())
}

fn sff_server(listener: TcpListener, queue: Channel) -> Result<()> {
    while let Ok((stream, _)) = listener.accept() {
        let Ok(path) = read_stream(&stream) else {
            eprintln!("error when parsing http");
            continue;
        };
        eprintln!("parsed http: {}", path.display());

        let Ok(size) = std::fs::metadata(&path).map(|meta| meta.size()) else {
            eprintln!("error when getting metadata");
            continue;
        };

        let job = Job {
            client: stream,
            path: Some(path),
            file_size: Some(size),
        };

        // TODO: enqueue algorithm
    }
    Ok(())
}

// specifies the amoount of workers, does not exceed the system's available cores
fn worker_count(count: usize) -> usize {
    let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if n < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to get thread count from sysconf {err}")
    }
    usize::min(n as usize, count)
}

#[cfg(test)]
mod test {
    #[test]
    fn cpu_count() {
        let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        assert_eq!(n, 24);
    }
}
