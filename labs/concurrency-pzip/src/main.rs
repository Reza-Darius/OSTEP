use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;

use concurrency_pzip::error::Result;
use concurrency_pzip::{FileMap, compv, stitch};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("wzip: file1 [file2 ...]");
        std::process::exit(1)
    }

    let files: Vec<FileMap> = args
        .iter()
        .map(FileMap::new)
        .collect::<Result<Vec<FileMap>>>()?;

    mt_dispatch(files.into_iter(), 10)
}

fn st_dispatch(files: impl Iterator<Item = FileMap>) -> Result<()> {
    let results: Vec<Vec<u8>> = files.map(|v| compv(v.as_slice())).collect();
    let r = stitch(results);

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&r)?;
    stdout.flush()?;

    Ok(())
}

fn mt_dispatch(files: impl Iterator<Item = FileMap>, part_factor: u8) -> Result<()> {
    // threshhold in bytes at which a file gets split among multiple threads
    const THRESHHOLD: usize = 1000;
    let part_factor = u8::max(1, part_factor);

    let results = std::thread::scope(move |s| {
        let mut handles = Vec::new();

        for file in files {
            if file.len() >= THRESHHOLD {
                // len = 107
                // part_factor = 2
                // workset_len = 53, remainder = 1
                // loop: n = 0, offset = 0 * workset_len = 0
                // worker: slice[0..0 + 53]
                //
                // remainder thread after loop:
                // slice[53..0]
                let workset_len = file.len() / part_factor as usize;
                let file = Arc::new(file);

                for n in 0..part_factor - 1 {
                    let f_clone = file.clone();
                    let offset = n as usize * workset_len;

                    handles.push(s.spawn(move || {
                        let slice = &f_clone.as_slice()[offset..offset + workset_len];
                        compv(slice)
                    }));
                }

                // the last work set has to account for remainder
                let offset = (part_factor as usize - 1) * workset_len;
                handles.push(s.spawn(move || {
                    let slice = &file.as_slice()[offset..];
                    compv(slice)
                }));
            } else {
                handles.push(s.spawn(move || {
                    let slice = file.as_slice();
                    compv(slice)
                }));
            }
        }
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<Vec<u8>>>()
    });

    let r = stitch(results);

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&r)?;
    stdout.flush()?;

    Ok(())
}
