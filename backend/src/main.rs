//mod nodb;
//mod note_db;
mod config;
mod my_logger;
use crate::config::*;
use crate::my_logger::*;
use std::env;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn parse_http_header(buf_reader: BufReader<&mut TcpStream>) -> Result<HttpHeader, &str> {
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    info!("Request: {http_request:#?}");
    if http_request.len() == 0 {
        return Err("Empty");
    }
    let l1: Vec<_> = http_request[0].split(' ').collect();
    if l1.len() < 3 {
        return Err("Error parsing first line");
    }

    return Ok(HttpHeader {
        method:   l1[0].to_string(),
        path:     l1[1].to_string(),
        version:  l1[2].to_string(),
    });
}

struct HttpHeader {
    method: String,
    path: String,
    version: String,
}

fn handle_connection(mut stream: TcpStream, frontend_dir: String) {
    let buf_reader = BufReader::new(&mut stream);
    let header = match parse_http_header(buf_reader) {
        Ok(h) => h,
        Err(e) => {
            warn!("HTTP Header error: {}", e);
            http_respond_error(stream);
            return;
        },
    };
    if header.method != "GET" {
        warn!("Unsupported method: {}", header.method);
        http_respond_error(stream);
        return;
    }


    http_respond_file(stream, frontend_dir + &header.path);
}

fn http_respond_error(mut stream: TcpStream) {
    let response = "HTTP/1.1 404 NOT FOUND\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}

fn http_respond_file(mut stream: TcpStream, fp: String) {
    // TODO: Check permissions
    let result = fs::read(fp.clone());
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            warn!("fs::read({}) - {}", fp, e);
            http_respond_error(stream);
            return;
        }
    };
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() <= 1 {
        println!("Expected <config.json> argument");
        return;
    }
    let config_fp = args[1].clone();
    let cfg = match parse_config_file(&config_fp) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };
    my_logger::init();
    println!("{:#?}", cfg);
    let listener = match TcpListener::bind(&cfg.address) {
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
        handle_connection(stream, cfg.frontend_dir.clone());
    }
}
