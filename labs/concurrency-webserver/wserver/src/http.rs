#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

const SP: u8 = 0x20;
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;
const CRLF: [char; 2] = ['\r', '\n'];

const HTTP_VERSION: &str = "HTTP/1.0";

pub fn write_response(data: &[u8], mut stream: impl Write) -> Result<()> {
    write!(
        stream,
        "
        {HTTP_VERSION} 200 OK\r\n\
        Content-Type: application/octet-stream\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
    ",
        data.len()
    )?;
    stream.write_all(data).map_err(Into::into)
}

pub fn parse_stream(stream: impl Read) -> Result<PathBuf> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();

    if reader.read_line(&mut buf)? == 0 {
        return Err(anyhow!("mangled request"));
    }

    let path = parse_request_line(&buf)?;

    // TODO: path security parsing

    let mut saw_delimiter = false;

    buf.clear();
    while reader.read_line(&mut buf)? != 0 {
        // TODO: header parsing
        if buf == "\r\n" {
            saw_delimiter = true;
            break;
        }
        buf.clear();
    }

    if !saw_delimiter {
        return Err(anyhow!("missing body delimiter"));
    }

    Ok(path)
}

fn parse_request_line(line: &str) -> Result<PathBuf> {
    if !line.ends_with(CRLF) {
        return Err(anyhow!("couldnt parse request line {line}"));
    }

    // request-line   = method SP request-target SP HTTP-version
    let mut req_line_iter = line.split_whitespace();

    let Some(method) = req_line_iter.next() else {
        return Err(anyhow!("couldnt parse request line"));
    };

    if method != "GET" {
        return Err(anyhow!("unsupported HTTP method {method}"));
    }

    let path = req_line_iter
        .next()
        .ok_or_else(|| anyhow!("failed to retrieve request target"))
        .map(PathBuf::from)?;

    let Some(version) = req_line_iter.next() else {
        return Err(anyhow!("couldnt retrieve version from req line"));
    };

    if version != HTTP_VERSION {
        return Err(anyhow!("unsupported HTTP version {version}"));
    }
    Ok(path)
}

fn parse_header(line: &str) -> Result<()> {
    if !line.ends_with(CRLF) {
        return Err(anyhow!("invalid header {line}"));
    }

    if !line.contains(':') {
        return Err(anyhow!("invalid header field"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn request_parse() {
        let request = "GET / HTTP/1.0\r\n\
        Host: example.com\r\n\
        User-Agent: test-client/1.0\r\n\
        Accept: */*\r\n\
        Connection: close\r\n\r\n";

        let r = parse_stream(request.as_bytes()).unwrap();
        assert_eq!("/", r.as_path());

        let request = "GET / HTTP/1.0\r\n\
        Host: example.com\r\n\
        User-Agent: test-client/1.0\r\n\
        Accept: */*\r\n\
        Connection: close\r\n";
        assert!(parse_stream(request.as_bytes()).is_err());

        let request = "GET / HTTP/1.0\r\n";
        assert!(parse_stream(request.as_bytes()).is_err());
    }
}
