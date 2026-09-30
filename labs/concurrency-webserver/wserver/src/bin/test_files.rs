use std::os::unix::fs::MetadataExt;

const SMALL: usize = 1 << 10;
const MEDIUM: usize = 1 << 20;
const LARGE: usize = 1 << 25;

fn main() {
    std::thread::scope(|s| {
        s.spawn(|| {
            eprintln!("started file1");

            let path = "files/file1.txt";
            let data = vec![b'A'; SMALL];
            std::fs::write(path, &data).unwrap();
            let size = std::fs::metadata(path).unwrap();
            eprintln!("wrote {} bytes in small file", size.size());
        });

        s.spawn(|| {
            eprintln!("started file2");

            let path = "files/file2.txt";
            let data = vec![b'A'; MEDIUM];
            std::fs::write(path, &data).unwrap();
            let size = std::fs::metadata(path).unwrap();
            eprintln!("wrote {} bytes in medium file", size.size());
        });

        s.spawn(|| {
            eprintln!("started file3");

            let path = "files/file3.txt";
            let data = vec![b'A'; LARGE];
            std::fs::write(path, &data).unwrap();
            let size = std::fs::metadata(path).unwrap();
            eprintln!("wrote {} bytes in large file", size.size());
        });
    });
}
