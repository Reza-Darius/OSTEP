use crate::{
    chan::Channel,
    handler::{FifoJob, Policy, SffJob, fifo_worker, sff_worker},
    http::read_stream,
};
use anyhow::Result;
use tracing::info;
use std::{net::TcpListener, os::unix::fs::MetadataExt, path::Path};

pub fn run(
    basedir: &'static Path,
    port: u16,
    threads: usize,
    buf_size: usize,
    schedule: Policy,
) -> Result<()> {
    let w_count = worker_count(threads);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)?;

    info!(
        "listening on {addr}, basedir: {}, with: {w_count} threads, buf_size: {}, schedule: {}",
        std::path::absolute(basedir)?.display(),
        buf_size,
        schedule
    );

    match schedule {
        Policy::Fifo => {
            let queue = Channel::new(buf_size);
            for _ in 0..w_count {
                let q = queue.clone();
                std::thread::spawn(move || fifo_worker(basedir, q));
            }

            while let Ok((stream, _)) = listener.accept() {
                queue.push_back(FifoJob { client: stream });
            }
        }
        Policy::Sff => {
            let queue = Channel::<SffJob>::new(buf_size);
            for _ in 0..w_count {
                let q = queue.clone();
                std::thread::spawn(move || sff_worker(basedir, q));
            }

            let mut job_counter = 0;
            while let Ok((stream, _)) = listener.accept() {
                let Ok(path) = read_stream(&stream) else {
                    eprintln!("error when parsing http");
                    continue;
                };

                let Ok(size) = std::fs::metadata(&path).map(|meta| meta.size()) else {
                    eprintln!("error when getting metadata");
                    continue;
                };

                let job = SffJob {
                    client: stream,
                    path,
                    file_size: size,
                };

                queue.enqueue_shortest_job(job);
            }
        }
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
