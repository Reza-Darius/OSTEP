use std::{
    error::Error, io::{Read, Write, stdin, stdout}, mem,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let num_iterations: u32 = args[0].parse::<u32>()?;

    // we pin the process to a CPU core to reduce any measurement noise caused by cache
    // invalidiation
    unsafe {
        let mut set: libc::cpu_set_t = mem::zeroed();
        libc::CPU_SET(1, &mut set);
        let pid = libc::getpid();
        if libc::sched_setaffinity(pid, mem::size_of::<libc::cpu_set_t>(), &mut set as *const _)
            == -1
        {
            let err = std::io::Error::last_os_error();
            eprintln!("failed to set cpu affinity {}", err);
            return Err(err.into());
        }
    }

    let mut reader = stdin().lock();
    let mut writer = stdout().lock();

    let mut buf = [0; 1];
    for _ in 0..num_iterations {
        let _ = reader.read(&mut buf);
        let _ = writer.write(&buf);
        writer.flush();
    }

    Ok(())
}
