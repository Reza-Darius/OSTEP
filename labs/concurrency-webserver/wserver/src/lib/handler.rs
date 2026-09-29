use std::{
    fmt::Display,
    net::TcpStream,
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::PathBuf,
};

use crate::{chan::Channel, http::read_stream};
use anyhow::Result;
use clap::ValueEnum;

pub struct Job {
    pub client: TcpStream,
    pub path: Option<PathBuf>,
    pub file_size: Option<u64>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
#[value(rename_all = "upper")]
pub enum Policy {
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

pub fn worker(queue: Channel) {
    loop {
        let job = queue.pop_front();
        if let Err(e) = handle_job(job) {
            eprintln!("couldnt handle job: {e}");
        };
    }
}

pub fn handle_job(job: Job) -> Result<()> {
    let path = if let Some(path) = job.path {
        path
    } else {
        read_stream(&job.client)?
    };

    let filesize = if let Some(size) = job.file_size {
        size as usize
    } else {
        std::fs::metadata(&path).map(|meta| meta.size() as usize)?
    };

    let file = std::fs::File::open(path)?;
    let mut n_sent = 0;

    unsafe {
        while n_sent < filesize {
            let rc = libc::sendfile(
                job.client.as_raw_fd(),
                file.as_raw_fd(),
                std::ptr::null_mut(),
                filesize - n_sent,
            );
            if rc == -1 {
                return Err(std::io::Error::last_os_error().into());
            }
            n_sent += rc as usize;
        }
    }
    Ok(())
}
