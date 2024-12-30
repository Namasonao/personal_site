use crate::http::types::*;
use std::net::TcpStream;
use std::io::{Error, Write, BufReader, Read, BufRead};
use crate::{info, warn};

impl HttpResponse {
    pub fn respond(&self, stream: &mut TcpStream) -> Result<(), Error> {
        let mut response = self.version.clone();
        response += " ";
        use StatusCode::*;
        response += match &self.status_code {
            OK => "200 OK",
            NotFound => "404 NOT FOUND",
            NotImplemented => "501 NOT IMPLEMENTED",
            BadRequest => "400 BAD REQUEST",
            InternalError => "500 INTERNAL SERVER ERROR",
            _ => todo!(),
        };
        response += "\r\n";

        for (left, right) in self.fields.iter() {
            response += &left;
            response += ": ";
            response += &right;
            response += "\r\n";
        }
        response += "\r\n";
        stream.write(response.as_bytes())?;

        if let Some(body) = &self.body {
            stream.write(&body)?;
        }

        Ok(())
    }
}

fn parse_field(line: String) -> Option<Field> {
    let mut it = line.split(':');
    let key = it.next()?.to_string();
    let mut val = it.next()?.to_string();
    val.remove(0);
    Some((key, val))
}

pub fn parse_http(mut buf_reader: BufReader<&mut TcpStream>) -> Result<HttpRequest, &'static str> {
    let mut first_line = String::new();
    if let Err(_) = buf_reader.read_line(&mut first_line) {
        return Err("Empty request");
    }
    let l1: Vec<_> = first_line.split(' ').collect();
    if l1.len() != 3 {
        return Err("Error parsing first line");
    }
    let method = match l1[0] {
        "GET" => Method::Get,
        "POST" => Method::Post,
        _ => return Err("Unknown method"),
    };
    let path = l1[1].to_string();
    let version = l1[2].to_string();
    let mut fields: Vec<Field> = Vec::new();
    let mut l = String::new();
    while let Ok(_) = buf_reader.read_line(&mut l) {
        let _ = l.pop();
        let _ = l.pop();
        if l.is_empty() {
            break;
        }
        if let Some(f) = parse_field(l.clone()) {
            fields.push(f);
        }
        l.clear();
    }

    info!("{:?} {} {}", method, path, version);

    let mut http_body = Vec::new();
    if method == Method::Post {
        let mut exp_len = 0;
        for f in &fields {
            if f.0 == "Content-Length" {
                exp_len = match f.1.parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => {
                        info!("Unexpected Content-Length");
                        0
                    }
                };
                break;
            }
        }
        http_body.reserve(exp_len);
        unsafe {
            http_body.set_len(exp_len);
        }
        if let Err(e) = buf_reader.read_exact(&mut http_body) {
            warn!("{}", e);
        }
    }

    return Ok(HttpRequest {
        method: method,
        path: path,
        version: version,
        fields: fields,
        body: Some(http_body),
    });
}
