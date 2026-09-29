use std::{net::TcpStream, path::PathBuf};

use anyhow::Result;
use crate::chan::Channel;

pub struct Job {
    pub client: TcpStream,
    pub file: PathBuf,
}

impl Job {
    pub fn new(client: TcpStream, path: PathBuf) -> Self {
        Job { client, file: path }
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
