use std::io::Write;

use concurrency_pzip::error::Result;
use concurrency_pzip::{FileMap, coalesce, compv};

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

    let mut results: Vec<Vec<u8>> = files.into_iter().map(|v| compv(v.as_slice())).collect();
    let mut r = results.remove(0);

    for res in results {
        r = coalesce(r, res);
    }

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&r)?;
    stdout.flush()?;
    Ok(())
}
