//mod nodb;
//mod note_db;
mod config;
mod my_logger;
use crate::config::*;
use log::{info, warn};
use std::env;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

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
    println!("{:#?}", net_cfg);
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
