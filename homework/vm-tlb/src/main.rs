#![allow(clippy::uninit_vec)]

use std::{env, error::Error, hint, mem, time::Duration};

use rand::random;

// we take two arguments:
// - amount of pages we want to touch
// - amount of iterations to test

const MAX_PAGES: usize = 62500 * 2;

fn main() -> Result<(), Box<dyn Error>> {
    let pagesize: usize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    let args = env::args()
        .skip(1)
        .take(2)
        .flat_map(|arg| arg.parse::<usize>())
        .collect::<Vec<usize>>();

    if args.is_empty() {
        return Err("invalid argument".into());
    };

    // we pin the process to a single CPU core to reduce any measurement noise caused by cache
    // invalidiation
    unsafe {
        let mut set: libc::cpu_set_t = mem::zeroed();
        libc::CPU_SET(0, &mut set);
        let pid = libc::getpid();
        if libc::sched_setaffinity(pid, mem::size_of::<libc::cpu_set_t>(), &mut set as *const _)
            == -1
        {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to set cpu affinity {}", err);
            return Err(err.into());
        }
    }

    let n_pages = args[0];
    let n_iterations = args[1];

    let mut n_pages = 4;
    while n_pages < MAX_PAGES {
        let res = with_mmap(n_pages, pagesize, n_iterations);
        println!("num pages: {}, average access time {:?}", n_pages, res);
        n_pages *= 2;
    }

    Ok(())
}

#[inline(never)]
fn with_mmap(n_pages: usize, pagesize: usize, n_iterations: usize) -> Duration {
    let map = new_map(n_pages, pagesize);

    // touch each page once for the kernel to page them in
    for i in 0..n_pages {
        unsafe {
            let ptr = map.add(i * pagesize);
            hint::black_box(*ptr += 1);
        }
    }

    let offsets = rand_offsets(n_pages, pagesize, n_iterations);

    let now = quanta::Instant::now();
    for offset in offsets {
        unsafe {
            let ptr = map.add(offset);
            let mem_read = hint::black_box(*ptr);
            hint::black_box(*ptr += 1);
        }
    }
    let elapsed = now.elapsed();

    close_map(map, n_pages, pagesize);
    elapsed.div_f64(n_iterations as f64)
}

fn rand_offsets(n_pages: usize, pagesize: usize, n_iterations: usize) -> Vec<usize> {
    Vec::from_fn(n_iterations, |_| {
        (random::<u64>() % (n_pages as u64 - 1)) as usize * pagesize
    })
}

fn new_map(n_pages: usize, pagesize: usize) -> *mut u8 {
    unsafe {
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            n_pages * pagesize,
            libc::PROT_WRITE | libc::PROT_READ,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );

        if ptr == libc::MAP_FAILED {
            eprintln!("mmap failed");
            std::process::exit(-1)
        } else {
            ptr as *mut _
        }
    }
}

fn close_map(map: *mut u8, n_pages: usize, pagesize: usize) {
    unsafe {
        if libc::munmap(map as *mut _, n_pages * pagesize) == -1 {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to unmap {}", err);
            std::process::exit(-1)
        };
    }
}
