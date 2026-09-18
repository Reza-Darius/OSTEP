use error::Result;
use std::{
    ffi::c_void,
    fs::metadata,
    io::Read,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::fs::MetadataExt,
    },
    path::Path,
};

use crate::error::AppError;

mod error;

pub struct FileMap {
    file: OwnedFd,
    len: usize,
    map: *const u8,
}

impl FileMap {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file: OwnedFd = std::fs::File::open(path)?.into();
        let len = metadata(path)?.size() as usize;

        unsafe {
            let map = libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::MAP_PRIVATE,
                libc::PROT_READ,
                file.as_raw_fd(),
                0,
            );
            if map == libc::MAP_FAILED {
                return Err(AppError::new(std::io::Error::last_os_error()));
            }
            let rc = libc::madvise(map, len, libc::MADV_SEQUENTIAL);
            if rc == -1 {
                return Err(AppError::new(std::io::Error::last_os_error()));
            }

            Ok(FileMap {
                file,
                len,
                map: map as *const u8,
            })
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.map, self.len) }
    }
}

impl Drop for FileMap {
    fn drop(&mut self) {
        unsafe {
            let rc = libc::munmap(self.map as *mut c_void, self.len);
            if rc == -1 {
                eprintln!("filemap drop error {}", std::io::Error::last_os_error());
            }
        }
    }
}

pub fn coalesce(mut a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
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
    let cmp_arr = [char; LANES];
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

    s
}

#[cfg(test)]
mod test {
    use super::*;

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
}
