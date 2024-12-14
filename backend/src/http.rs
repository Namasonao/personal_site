use std::net::TcpStream;
use std::io::{BufReader, BufRead};
use crate::info;


pub fn parse_http_header(buf_reader: BufReader<&mut TcpStream>) -> Result<HttpHeader, &str> {
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
        //version:  l1[2].to_string(),
    });
}

pub struct HttpHeader {
    pub method: String,
    pub path: String,
    //pub version: String,
}
