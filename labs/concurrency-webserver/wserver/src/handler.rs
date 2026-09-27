use std::{net::TcpStream, path::PathBuf};

use anyhow::Result;
use schloss::channel::Channel;

pub struct Job {
    pub stream: TcpStream,
    pub file: PathBuf,
}

impl Job {
    pub fn new(stream: TcpStream, path: PathBuf) -> Self {
        Job { stream, file: path }
    }
}

pub fn worker(queue: Channel<Job>) {
    loop {
        let job = queue.pop_front();
        let _ = handle_job(job);
    }
}

pub fn handle_job(job: Job) -> Result<()> {
    println!("handling job {}", job.file.display());
    Ok(())
}
