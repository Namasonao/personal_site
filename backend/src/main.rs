mod nodb;
mod note_db;
use log::LevelFilter;
use log::{info, warn};
use log::{Level, Metadata, Record};
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    http_respond_file(stream, "./frontend/index.html".to_string());
}

fn http_respond_file(mut stream: TcpStream, fp: String) {
    // TODO: Check permissions
    let result = fs::read(fp.clone());
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            let response = "HTTP/1.1 404 NOT FOUND\r\n";
            stream.write_all(response.as_bytes()).unwrap();
            warn!("fs::read({}) - {}", fp, e);
            return;
        }
    };
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn main() {
    if let Err(e) = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info)) {
        println!("NO LOGGER");
    };
    println!("Hello");
    let listener = match TcpListener::bind("0.0.0.0:7878") {
        Ok(l) => l,
        Err(e) => panic!("{}", e),
    };

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                warn!("{}", e);
                continue;
            }
        };
        info!(
            "Connection established with {}!",
            stream.peer_addr().unwrap()
        );
        handle_connection(stream);
    }
}
