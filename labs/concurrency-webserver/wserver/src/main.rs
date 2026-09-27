/*
* HTTP file server over a threadpool
* 1. parse http
* 2. setup thread pool
* 3. setup server
*/

use anyhow::Result;
use clap::{Parser, ValueEnum};
use schloss::channel::Channel;
use std::{fmt::Display, net::TcpListener, path::PathBuf, thread};
use wserver::handler::Job;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'n', default_value = ".")]
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
    Sff,
    #[default]
    Fifo,
}

impl Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Sff => write!(f, "SFF"),
            Policy::Fifo => write!(f, "FIFO"),
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let w_count = worker_count(&args);
    let queue = Channel::<Job>::new(args.buffer);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))?;

    Ok(())
}

fn worker_count(args: &Cli) -> usize {
    let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if n < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to get thread count from sysconf {err}")
    }
    usize::min(n as usize, args.threads)
}

#[cfg(test)]
mod test {
    #[test]
    fn cpu_count() {
        let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        assert_eq!(n, 24);
    }
}
