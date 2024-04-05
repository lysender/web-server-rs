use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use tracing::info;

use crate::worker::WorkerPool;

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let worker_pool = WorkerPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        worker_pool.execute(|| {
            handle_connect(stream);
        });
    }

    info!("Shutting down.");
}

fn handle_connect(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Log the first line
    let lines: Vec<String> = String::from_utf8_lossy(&buffer[..])
        .lines()
        .map(|x| x.to_string())
        .collect();

    info!("{}", lines[0]);

    // Basic routing capability
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        std::thread::sleep(std::time::Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    handle_response(stream, status_line, filename);
}

fn handle_response(mut stream: TcpStream, status_line: &str, filename: &str) {
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
