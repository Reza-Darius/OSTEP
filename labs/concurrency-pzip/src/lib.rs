use error::Result;
use std::{ffi::CString, fs::metadata, os::unix::fs::MetadataExt, path::Path};

pub mod error;

pub struct FileMap {
    fd: i32,
    len: usize,
    map: *const u8,
}

// SAFETY: threads only have read access
unsafe impl Send for FileMap {}
unsafe impl Sync for FileMap {}

impl FileMap {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let len = metadata(&path)?.size() as usize;
        let path = CString::new(path.as_ref().to_str().unwrap())?;

        unsafe {
            let fd = libc::open(path.as_ptr(), libc::O_RDONLY);
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            let map = libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                fd,
                0,
            );
            if map == libc::MAP_FAILED {
                return Err(std::io::Error::last_os_error().into());
            }
            let rc = libc::madvise(map, len, libc::MADV_SEQUENTIAL | libc::MADV_WILLNEED);
            if rc == -1 {
                return Err(std::io::Error::last_os_error().into());
            }

            Ok(FileMap {
                fd,
                len,
                map: map as *const u8,
            })
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.map, self.len) }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for FileMap {
    fn drop(&mut self) {
        unsafe {
            let rc = libc::munmap(self.map as *mut _, self.len);
            if rc == -1 {
                eprintln!("filemap drop error {}", std::io::Error::last_os_error());
            }
            libc::close(self.fd);
        }
    }
}

pub fn stitch(mut data: Vec<Vec<u8>>) -> Vec<u8> {
    let mut r = data.remove(0);
    for res in data {
        r = coalesce(r, res);
    }
    r
}

fn coalesce(mut a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    debug_assert!(a.len() >= 5);
    debug_assert!(b.len() >= 5);

    // is the last letter of a the same and the first letter of b?
    if *a.last().unwrap() == b[4] {
        // take the count from b and a, add both to the end of a
        let offset = a.len() - 1 - 4;
        let a_count = u32::from_le_bytes(a[offset..offset + 4].try_into().unwrap());
        let b_count = u32::from_le_bytes(b[..4].try_into().unwrap());
        a[offset..offset + 4].copy_from_slice(&(a_count + b_count).to_le_bytes());

        // cut off the segment from b
        a.extend_from_slice(&b[5..]);
    } else {
        a.extend(b);
    }
    a
}

pub fn comps(input: &[u8]) -> Vec<u8> {
    let mut s = Vec::new();

    if input.is_empty() {
        return s;
    }

    let mut char = input[0];
    let mut count: u32 = 0;

    for byte in input.iter().copied() {
        if byte == char {
            count += 1;
        } else {
            s.extend_from_slice(&count.to_le_bytes());
            s.push(char);
            count = 1;
            char = byte;
        }
    }

    // add the last one
    s.extend_from_slice(&count.to_le_bytes());
    s.push(char);

    debug_assert!(s.len() % 5 == 0);
    s
}

pub fn compv(input: &[u8]) -> Vec<u8> {
    const LANES: usize = 16;
    let mut s = Vec::new();

    if input.is_empty() {
        return s;
    }

    let (chunks, remainder) = input.as_chunks::<LANES>();

    let mut char = input[0];
    let mut cmp_arr = [char; LANES];
    let mut count: u32 = 0;

    for chunk in chunks {
        if chunk == &cmp_arr {
            count += LANES as u32;
            continue;
        }
        for &byte in chunk {
            if byte == char {
                count += 1;
            } else {
                s.extend_from_slice(&count.to_le_bytes());
                s.push(char);
                count = 1;
                char = byte;
                cmp_arr = [char; LANES];
            }
        }
    }

    for byte in remainder.iter().copied() {
        if byte == char {
            count += 1;
        } else {
            s.extend_from_slice(&count.to_le_bytes());
            s.push(char);
            count = 1;
            char = byte;
        }
    }

    // add the last one
    s.extend_from_slice(&count.to_le_bytes());
    s.push(char);

    debug_assert!(s.len() % 5 == 0);
    s
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_file(input: &str) -> FileMap {
        const TEST_FILE: &str = "./test_file.txt";

        std::fs::write(TEST_FILE, input.as_bytes()).unwrap();
        FileMap::new(TEST_FILE).unwrap()
    }

    #[test]
    fn comp_test() {
        let input = "aaaaaaaaaabbbb";
        let mut res = Vec::new();

        res.extend_from_slice(&u32::to_le_bytes(10));
        res.push(b'a');
        res.extend_from_slice(&u32::to_le_bytes(4));
        res.push(b'b');

        assert_eq!(res, comps(input.as_bytes()));
        assert_eq!(res, compv(input.as_bytes()));
    }

    #[test]
    fn comp_test2() {
        let input = "aaaaaaaaaa";
        let mut res = Vec::new();

        res.extend_from_slice(&u32::to_le_bytes(10));
        res.push(b'a');

        assert_eq!(res, comps(input.as_bytes()));
        assert_eq!(res, compv(input.as_bytes()));
    }

    #[test]
    fn coalesce_test() {
        let input = "aaaaaaaaaabbbb";
        let input2 = "bbbbaaa";
        let mut res = Vec::new();

        // expected output: 10a8b3a
        res.extend_from_slice(&u32::to_le_bytes(10));
        res.push(b'a');
        res.extend_from_slice(&u32::to_le_bytes(8));
        res.push(b'b');
        res.extend_from_slice(&u32::to_le_bytes(3));
        res.push(b'a');

        let coal = coalesce(compv(input.as_bytes()), compv(input2.as_bytes()));
        assert_eq!(res, coal);
    }

    #[test]
    fn comp_test_file() {
        let input = "aaaaaaaaaabbbb";
        let file_map = test_file(input);

        let mut res = Vec::new();
        res.extend_from_slice(&u32::to_le_bytes(10));
        res.push(b'a');
        res.extend_from_slice(&u32::to_le_bytes(4));
        res.push(b'b');

        assert_eq!(res, comps(file_map.as_slice()));
        assert_eq!(res, compv(file_map.as_slice()));
    }
}
