/*
* HTTP file server over a threadpool
* 1. parse http
* 2. setup thread pool
* 3. setup server
*/

use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::{fmt::Display, net::TcpListener, path::PathBuf};
use wserver::{
    chan::Channel,
    handler::{Job, worker},
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
#[value(rename_all = "upper")]
enum Policy {
    #[default]
    Fifo,
    Sff,
}

impl Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Fifo => write!(f, "FIFO"),
            Policy::Sff => write!(f, "SFF"),
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let w_count = worker_count(args.threads);
    let queue = Channel::<Job>::new(args.buffer);
    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&addr)?;

    for _ in 0..w_count {
        let q = queue.clone();
        std::thread::spawn(|| worker(q));
    }

    eprintln!("listening on {addr}");

    match args.schedule {
        Policy::Fifo => fifo_server(listener, queue)?,
        Policy::Sff => todo!(),
    }

    Ok(())
}

fn fifo_server(listener: TcpListener, queue: Channel<Job>) -> Result<()> {
    while let Ok((stream, _)) = listener.accept() {
        let Ok(path) = read_stream(&stream) else {
            eprintln!("error when parsing http");
            continue;
        };
        eprintln!("parsed http: {}", path.display());

        queue.push_back(Job { client: stream, file: path });
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
