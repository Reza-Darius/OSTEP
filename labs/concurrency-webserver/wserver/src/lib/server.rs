use crate::{
    chan::Channel,
    handler::{Job, Policy, worker},
    http::read_stream,
};
use anyhow::Result;
use std::{net::TcpListener, os::unix::fs::MetadataExt, path::Path};

pub fn run(
    basedir: &'static Path,
    port: u16,
    threads: usize,
    buffer: usize,
    schedule: Policy,
) -> Result<()> {
    let w_count = worker_count(threads);
    let queue = Channel::new(buffer);
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)?;

    for _ in 0..w_count {
        let q = queue.clone();
        std::thread::spawn(move || worker(basedir, q));
    }

    eprintln!(
        "listening on {addr}, basedir: {}, with: {w_count} threads, buffer: {}, schedule: {}",
        std::path::absolute(basedir)?.display(),
        buffer,
        schedule
    );

    match schedule {
        Policy::Fifo => fifo_server(listener, queue)?,
        Policy::Sff => sff_server(listener, queue)?,
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

        queue.enqueue_shortest_job(job);
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
