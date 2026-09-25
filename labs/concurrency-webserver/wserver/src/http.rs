#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub struct Request {
    pub file: PathBuf,
}

const SP: u8 = 0x20;
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

pub fn parse_http(buf: impl AsRef<[u8]>) -> Result<Request> {
    let buf = buf.as_ref();

    // split at new line
    let mut line_iter = buf.split(|&e| e == LF);

    let req_line = line_iter
        .next()
        .ok_or_else(|| anyhow!("couldnt parse request line"))
        .and_then(check_cr)?;

    let path = parse_request_line(req_line)?;
    let mut saw_delimiter = false;

    for line in line_iter.map(check_cr) {
        let line = line?;
        if line.is_empty() {
            saw_delimiter = true;
            break
        }
    }

    if !saw_delimiter {
        return Err(anyhow!("missing body delimiter"))
    }

    Ok(Request { file: path })
}

// trim off CR
fn check_cr(line: &[u8]) -> Result<&[u8]> {
    if line.last().copied().ok_or_else(|| anyhow!("empty line"))? != CR {
        return Err(anyhow!("couldnt parse request line"));
    }
    let line = &line[..line.len() - 1];
    Ok(line)
}

fn parse_request_line(line: &[u8]) -> Result<PathBuf> {
    // request-line   = method SP request-target SP HTTP-version
    let mut req_line_iter = line.split(|&e| e == SP);

    let Some(method) = req_line_iter.next() else {
        return Err(anyhow!("couldnt parse request line"));
    };

    if method != "GET".as_bytes() {
        return if let Ok(m) = std::str::from_utf8(method) {
            Err(anyhow!("unsupported HTTP method {m}"))
        } else {
            Err(anyhow!("unsupported HTTP method"))
        };
    }

    let path = req_line_iter
        .next()
        .ok_or_else(|| anyhow!("failed to retrieve request target"))
        .and_then(|path| String::from_utf8(path.to_vec()).map_err(Into::into))
        .map(PathBuf::from)?;

    let Some(version) = req_line_iter.next() else {
        return Err(anyhow!("couldnt retrieve version from req line"));
    };

    if version != "HTTP/1.1".as_bytes() {
        return if let Ok(v) = std::str::from_utf8(version) {
            Err(anyhow!("unsupported HTTP version {v}"))
        } else {
            Err(anyhow!("unsupported HTTP version"))
        };
    }
    Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn request_parse() {
        let request = "GET / HTTP/1.1\r\n\
        Host: example.com\r\n\
        User-Agent: test-client/1.0\r\n\
        Accept: */*\r\n\
        Connection: close\r\n\r\n";

        let r = parse_http(request.as_bytes()).unwrap();
        assert_eq!("/", r.file.as_path());

        let request = "GET / HTTP/1.1\r\n\
        Host: example.com\r\n\
        User-Agent: test-client/1.0\r\n\
        Accept: */*\r\n\
        Connection: close\r\n";
        assert!(parse_http(request.as_bytes()).is_err());

        let request = "GET / HTTP/1.1\r\n";
        assert!(parse_http(request.as_bytes()).is_err());
    }
}
