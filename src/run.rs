use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;
use tracing::info;

use crate::worker::WorkerPool;

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let worker_pool = WorkerPool::new(4);

    info!("Server started on port 3000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        worker_pool.execute(|| {
            handle_connect(stream);
        });
    }

    info!("Shutting down.");
}

fn handle_connect(stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let request: Vec<_> = reader
        .lines()
        .map(|x| x.unwrap())
        .take_while(|x| !x.is_empty())
        .collect();

    let Some(first_line) = request.first() else {
        return handle_invalid_request(stream);
    };

    info!("{}", first_line);

    let route_parts: Vec<&str> = first_line.split_whitespace().collect();
    let Some(method) = route_parts.get(0) else {
        return handle_invalid_request(stream);
    };

    let Some(uri) = route_parts.get(1) else {
        return handle_invalid_request(stream);
    };

    let (status_line, filename) = match *uri {
        "/" => ("HTTP/1.1 200 OK", "index.html"),
        "/sleep" => {
            std::thread::sleep(std::time::Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
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

fn handle_invalid_request(mut stream: TcpStream) {
    let contents = fs::read_to_string("400.html").unwrap();
    let response = format!(
        "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
