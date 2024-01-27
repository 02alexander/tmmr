use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const CLEAR_PREV_LINE: &str = "\x1b[1A\x1b[2K\x1b[0G";
const SET_RED: &str = "\x1b[30;41";
const CLEAR_COLOR: &str = "\x1b[0m";

/// Returns (hours, minute, seconds)
fn parse_request(request: &[u8]) -> Option<(u32, u32, u32)> {
    let input_str = std::str::from_utf8(&request).ok()?;
    let first_line = input_str.lines().next()?;
    let url = first_line.split_whitespace().skip(1).next()?;
    let inp = url.split('/').skip(1).next()?;
    
    let mut times = inp.split(':').rev().map(|s| s.parse::<u32>().ok());
    let seconds = times.next().flatten().unwrap_or(0);
    let minutes = times.next().flatten().unwrap_or(0);
    let hours = times.next().flatten().unwrap_or(0);
    Some((hours, minutes, seconds))
}

fn pad_left(s: &str, n: usize, ch: char) -> String {
    let n = if n < s.len() { s.len() } else {n};
    String::from_iter(std::iter::repeat(ch).take(n-s.len()).chain(s.chars()))
}

async fn handle_request(mut stream: TcpStream) -> Result<(), Error> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).await?;

    let (hours, minutes, seconds) =
        parse_request(&buffer).ok_or(Error::from(ErrorKind::InvalidData))?;
    let tot_seconds = seconds+minutes*60+hours*3600;

    let status = b"HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(status).await?;
    for sec in (0..=tot_seconds).rev() {
        if sec != tot_seconds {
            stream.write_all(CLEAR_PREV_LINE.as_bytes()).await?;
        }
        let mut s = String::new();
        if sec >= 3600 {
            s += &pad_left(&(sec/3600).to_string(), 2, '0');
            s += ":";
        }
        if sec >= 60 {
            s += &pad_left(&((sec/60) % 60).to_string(), 2, '0');
            s += ":";
        }
        s += &pad_left(&(sec % 60).to_string(), 2, '0');
        s += "\n";

        if sec <= 10 {
            stream.write_all(SET_RED.as_bytes()).await?;
        }
        stream.write_all(s.as_bytes()).await?;
        if sec <= 10 {
            stream.write_all(CLEAR_COLOR.as_bytes()).await?;
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    stream.flush().await
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    loop {
        let (stream, source) = listener.accept().await.unwrap();
        println!("received connection from {source:?}");
        tokio::spawn(async move {
            if let Err(e) = handle_request(stream).await {
                eprintln!("Error handling request: {e:?}");
            }
        });
    }
}
