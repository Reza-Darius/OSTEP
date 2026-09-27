use std::{net::TcpStream, path::PathBuf};

use anyhow::Result;
use schloss::channel::Channel;

pub struct Job {
    stream: TcpStream,
    file: PathBuf,
}

pub fn worker(queue: Channel<Job>) {
    loop {
        let job = queue.pop_front();
        let _ = handle_job(job);
    }
}

pub fn handle_job(job: Job) -> Result<()> {
    todo!()
}
