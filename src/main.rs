use std::result::{Result};
use std::net::{TcpStream};
use std::io::{BufRead, BufReader, Read, Write};
use sha2::{Sha256, Digest};

#[derive(Debug)]
enum DlErr {
    ConErr,
    ReqErr,
    ResErr,
}

type Chunk = (Vec<u8>, usize);

fn dl_chunk(addr: &str, start: usize, end: usize) -> Result<Chunk, DlErr>{
    let mut stream = TcpStream::connect(addr).map_err(|_| DlErr::ConErr)?;

    let req = format!(
        "GET / HTTP/1.1\r\n\
        Host: {}\r\n\
        Range: bytes={}-{}\r\n\
        Connection: close\r\n\r\n",
        addr, start, end
    );

    println!("[REQ] Range: {} - {}", start, end);

    stream.write_all(req.as_bytes()).map_err(|_| DlErr::ReqErr)?;
    stream.flush().map_err(|_| DlErr::ReqErr)?;

    let mut buf_r = BufReader::new(stream);
    let mut exp_size: Option<usize> = None;

    loop {
        let mut line = String::new();
        let _ = buf_r.read_line(&mut line);

        if line == "\r\n" { break; }

        if let Some(val) = line
            .strip_prefix("Content-Length:")
            .and_then(|x| x.trim().parse::<usize>().ok()) 
        {
            exp_size = Some(val);
            println!("[RES] Length (expect): {}", val);
        }
    }

    let mut data: Vec<u8> = Vec::new();
    let size = buf_r.read_to_end(&mut data).map_err(|_| DlErr::ResErr)?;

    println!("[RES] Lenght (actual): {}\n", size);

    if let Some(exp) = exp_size {
        if size == 0 && exp > 0 {
            return Err(DlErr::ResErr);
        }
    }

    Ok((data, size))
}


fn main() -> Result<(), DlErr> {
    let srv_url: &str = "127.0.0.1:8080";
    const CHUNK_SIZE: usize = 100 * 1024;

    let mut start = 0;
    let mut end = CHUNK_SIZE;
    let mut data: Vec<u8> = Vec::new();

    loop {
        match dl_chunk(srv_url, start, end) {
            Ok((cdata, csize)) => {
                if csize == 0 { break };
                data.extend(cdata);
                start += csize;
                end = start + CHUNK_SIZE;
            },
            Err(DlErr::ConErr) => {
                println!("[ERR] Connectin error");
                return Err(DlErr::ConErr);
            },
            Err(DlErr::ReqErr) => {
                println!("[ERR] Request error. Trying again...");
                continue;
            },
            Err(DlErr::ResErr) => {
                println!("[ERR] Response error. Trying again...");
                continue;
            }
        }
    }

    let mut sha = Sha256::new();
    sha.update(&data);
    let hash = format!("{:x}", sha.finalize());

    println!("Length of the data: {}", data.len());
    println!("SHA-256 hash of the data: {}", hash);

    Ok(())
}
