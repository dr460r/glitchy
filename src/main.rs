use std::error::{Error};
use std::result::{Result};
use std::net::{TcpStream};
use std::io::{BufRead, BufReader, Read, Write};
use sha2::{Sha256, Digest};


fn dl_chunk(addr: &str, start: usize, end: usize) -> Result<(Vec<u8>, usize), Box<dyn Error>>{
    let mut stream = TcpStream::connect(addr)?;

    let req = format!(
        "GET / HTTP/1.1\r\n\
        Host: {}\r\n\
        Range: bytes={}-{}\r\n\
        Connection: close\r\n\r\n",
        addr, start, end
    );

    println!("[REQ] Range: {} - {}", start, end);

    stream.write_all(req.as_bytes())?;
    stream.flush()?;

    let mut buf_r = BufReader::new(stream);

    loop {
        let mut line = String::new();
        let _ = buf_r.read_line(&mut line);
        if line == "\r\n" {
            break;
        }
        if line.to_lowercase().starts_with("content-length:") {
            if let Some(cl_val) = line.split(':').nth(1) {
                let cnt_len_head = cl_val.trim().parse::<usize>()?;
                println!("[RES] Length (expect): {}", cnt_len_head);
            }
        }
    }

    let mut data: Vec<u8> = Vec::new();
    let size = buf_r.read_to_end(&mut data)?;

    println!("[RES] Lenght (actual): {}\n", size);

    Ok((data, size))
}


fn main() -> Result<(), Box<dyn Error>> {
    let srv_url: &str = "127.0.0.1:8080";
    const CHUNK_SIZE: usize = 100000;

    let mut start = 0;
    let mut end = CHUNK_SIZE;
    let mut data: Vec<u8> = Vec::new();
    loop {
        let (chunk, size) = dl_chunk(srv_url, start, end)?;
        if size == 0 { break; }
        data.extend(chunk);
        start += size;
        end = start + CHUNK_SIZE;
    }

    let mut sha = Sha256::new();
    sha.update(&data);
    let hash = format!("{:x}", sha.finalize());

    println!("Length of the data: {}", data.len());

    println!("SHA-256 hash of the data: {}", hash);

    Ok(())
}
