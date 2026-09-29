use std::{
    fmt::Display,
    net::TcpStream,
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Path, PathBuf},
};

use crate::{
    chan::Channel,
    http::{read_stream, write_response},
};
use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug)]
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

pub fn worker(basedir: &Path, queue: Channel) {
    loop {
        let job = queue.pop_front();
        if let Err(e) = handle_job(basedir, job) {
            eprintln!("couldnt handle job: {e}");
        };
    }
}

fn handle_job(basedir: &Path, mut job: Job) -> Result<()> {
    let path = if let Some(path) = job.path {
        path
    } else {
        read_stream(&job.client)?
    };

    let path = basedir.join(path);

    let filesize = if let Some(size) = job.file_size {
        size as usize
    } else {
        std::fs::metadata(&path).map(|meta| meta.size() as usize)?
    };

    let file = std::fs::File::open(&path)?;
    let mut n_sent = 0;

    eprintln!("handling job: path: {}, size: {}", path.display(), filesize);

    write_response(&mut job.client, filesize)?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn path_test() {
        let p = Path::new(".");
        assert_eq!(p.join(Path::new("/foo.txt")), Path::new("/foo.txt"));
        // let p = std::path::absolute(Path::new(".")).unwrap();
        let p2 = p.join("foo");
        eprintln!("{}", p2.display());
    }
}
