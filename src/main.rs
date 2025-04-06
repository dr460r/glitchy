use std::error::{Error};
use std::result::{Result};
use std::net::{TcpStream};
use std::io::{BufRead, BufReader, Read, Write};
//use sha2::{Sha256, Digest};


fn dl_chunk(addr: &str, start: usize, end: usize) -> Result<(Vec<u8>, usize), Box<dyn Error>>{
    println!("[LOG] Stream to be created");

    let mut stream = TcpStream::connect(addr)?;

    println!("[LOG] Stream created");

    let req = format!(
        "GET / HTTP/1.1\r\n\
        Host: {}\r\n\
        Range: bytes={}-{}\r\n\
        Connection: close\r\n\r\n",
        addr, start, end
    );

    println!("[LOG] HEADER\n{}", req);

    stream.write_all(req.as_bytes())?;
    stream.flush()?;

    let mut buf_r = BufReader::new(stream);

    loop {
        let mut sbuf = String::new();
        let _ = buf_r.read_line(&mut sbuf);
        print!("[LOG] {}", sbuf);
        if sbuf == "\r\n" {
            break;
        }
    }

    let mut data: Vec<u8> = Vec::new();
    let size = buf_r.read_to_end(&mut data)?;

    println!("[LOG] DATA, size: {}\n{:?}", size, data);

    Ok((vec![], size))
}


fn main() -> Result<(), Box<dyn Error>> {
    let srv_url: &str = "127.0.0.1:8080";

    const CHUNK_SIZE: usize = 100000;
    let mut start = 0;
    let mut end = 100;
    loop {
        let mut data: Vec<u8> = Vec::new();
        let (chunk, size) = dl_chunk(srv_url, start, end)?;
        if size == 0 { break; }
        data.extend(chunk);
        start += size;
        end = start + CHUNK_SIZE;
    }

    Ok(())
}
