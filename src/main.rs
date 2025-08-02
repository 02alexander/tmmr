use std::fmt::Write;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const CLEAR_PREV_LINE: &str = "\x1b[1A\x1b[2K\x1b[1G";
const SET_RED: &str = "\x1b[37;41m";
const CLEAR_COLOR: &str = "\x1b[0m";
const ALARM_TEXT: &str = "
  AAA   LL        AAA   RRRRRR  MM    MM 
 AAAAA  LL       AAAAA  RR   RR MMM  MMM 
AA   AA LL      AA   AA RRRRRR  MM MM MM 
AAAAAAA LL      AAAAAAA RR  RR  MM    MM 
AA   AA LLLLLLL AA   AA RR   RR MM    MM ";

/// Returns (hours, minute, seconds)
fn parse_request(request: &[u8]) -> Option<(u32, u32, u32)> {
    let input_str = std::str::from_utf8(request).ok()?;
    let first_line = input_str.lines().next()?;
    let url = first_line.split_whitespace().nth(1)?;
    let inp = url.split('/').nth(1)?;

    let mut times = inp.split(':').rev().map(|s| s.parse::<u32>().ok());
    let seconds = times.next().unwrap_or(Some(0))?;
    let minutes = times.next().unwrap_or(Some(0))?;
    let hours = times.next().unwrap_or(Some(0))?;
    Some((hours, minutes, seconds))
}

fn format_time(sec: u32) -> String {
    let mut s = String::new();
    if sec >= 3600 {
        write!(&mut s, "{:0>2}:", sec / 3600).unwrap();
    }
    if sec >= 60 {
        write!(&mut s, "{:0>2}:", (sec / 60) % 60).unwrap();
    }
    write!(&mut s, "{:0>2}", sec % 60).unwrap();
    s
}

async fn handle_request(stream: &mut TcpStream) -> Result<(), Error> {
    let mut buffer = [0_u8; 4196];
    let n = stream.read(&mut buffer).await?;
    if let Ok(s) = std::str::from_utf8(&buffer[..n]) {
        println!("Received request to the following url");
        println!("{:?}", s.lines().next());
    }

    let (hours, minutes, seconds) =
        parse_request(&buffer[..n]).ok_or(Error::from(ErrorKind::InvalidData))?;
    let tot_seconds = seconds + minutes * 60 + hours * 3600;

    let status = b"HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(status).await?;
    for sec in (0..=tot_seconds).rev() {
        if sec != tot_seconds {
            stream.write_all(CLEAR_PREV_LINE.as_bytes()).await?;
        }
        let s = format_time(sec);
        let red_threshold = 5;
        if sec <= red_threshold {
            stream.write_all(SET_RED.as_bytes()).await?;
        }
        stream.write_all(s.as_bytes()).await?;
        if sec <= red_threshold {
            stream.write_all(CLEAR_COLOR.as_bytes()).await?;
        }
        stream.write_all(b"\n").await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    stream.write_all(CLEAR_PREV_LINE.as_bytes()).await?;
    stream.write_all(SET_RED.as_bytes()).await?;
    stream.write_all(ALARM_TEXT.as_bytes()).await?;
    stream.write_all(CLEAR_COLOR.as_bytes()).await?;
    stream.write_all(b"\x07\n").await?;
    stream.flush().await
}

#[tokio::main]
async fn main() {
    let port = std::env::args().nth(1).unwrap_or("8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        if let Ok((mut stream, source)) = listener.accept().await {
            println!("received connection from {source:?}");
            tokio::spawn(async move {
                match handle_request(&mut stream).await {
                    Ok(_) => {}
                    Err(e) => match e.kind() {
                        ErrorKind::InvalidData => {
                            println!("Invalid data");
                            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nUsage: curl ip:8080/<hours>:<minutes>:<seconds>\r\nExample curl ip:8080/1:15:0\r\n").await;
                        }
                        _ => eprintln!("Error handling request: {e:?}"),
                    },
                }
            });
        } else {
            eprintln!("Error accepting incoming connection.");
        }
    }
}
