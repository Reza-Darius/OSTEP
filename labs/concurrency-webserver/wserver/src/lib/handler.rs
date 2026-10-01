use std::{
    fmt::Display, fs::File, net::TcpStream, os::{fd::AsRawFd, unix::fs::MetadataExt}, path::{Path, PathBuf}, thread, time::Duration,
};

use crate::{
    chan::Channel,
    http::{read_stream, write_http_resp},
};
use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug)]
pub struct FifoJob {
    pub client: TcpStream,
}

#[derive(Debug)]
pub struct SffJob {
    pub client: TcpStream,
    pub path: PathBuf,
    pub file_size: u64,
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
pub fn sff_worker(basedir: &Path, queue: Channel<SffJob>) {
    loop {
        // thread::sleep(Duration::from_secs(1));
        let job = queue.pop_front();
        println!("got job");
        if let Err(e) = handle_sff_job(basedir, job) {
            eprintln!("couldnt handle job: {e}");
        };
        println!("job done");
    }
}

fn handle_sff_job(basedir: &Path, job: SffJob) -> Result<()> {
    let path = basedir.join(job.path);
    let file = std::fs::File::open(&path)?;

    // eprintln!(
    //     "handling sff job: path: {}, size: {}",
    //     path.display(),
    //     job.file_size
    // );

    write_response(&file, job.file_size as usize, &job.client)?;
    Ok(())
}

pub fn fifo_worker(basedir: &Path, queue: Channel<FifoJob>) {
    loop {
        let job = queue.pop_front();
        if let Err(e) = handle_fifo_job(basedir, job) {
            eprintln!("couldnt handle job: {e}");
        };
    }
}

fn handle_fifo_job(basedir: &Path, job: FifoJob) -> Result<()> {
    let req_path = read_stream(&job.client)?;
    let path = basedir.join(req_path);

    let file_size = std::fs::metadata(&path).map(|meta| meta.size() as usize)?;
    let file = std::fs::File::open(&path)?;

    // eprintln!(
    //     "handling fifo job: path: {}, size: {}",
    //     path.display(),
    //     file_size
    // );

    write_response(&file, file_size, &job.client)?;
    Ok(())
}

fn write_response(file: &File, file_size: usize, client: &TcpStream) -> Result<()> {
    let mut n_sent = 0;

    write_http_resp(client, file_size)?;
    unsafe {
        while n_sent < file_size {
            let rc = libc::sendfile(
                client.as_raw_fd(),
                file.as_raw_fd(),
                std::ptr::null_mut(),
                file_size - n_sent,
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
