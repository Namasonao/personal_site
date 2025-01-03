use crate::http::types::*;
use crate::{info, warn};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::os::fd::{AsFd, BorrowedFd};

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
    if let Err(e) = buf_reader.read_line(&mut first_line) {
        warn!("Error reading line: {}", e.kind());
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
        // ignore \r\n
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
        info!("Reading body with expected size of {} bytes", exp_len);
        http_body.reserve(exp_len);
        unsafe {
            http_body.set_len(exp_len);
        }
        if let Err(e) = buf_reader.read_exact(&mut http_body) {
            warn!("{}", e);
        }
        info!("Read the following: {:?}", http_body);
    }

    return Ok(HttpRequest {
        method: method,
        path: path,
        version: version,
        fields: fields,
        body: Some(http_body),
    });
}

#[derive(Clone)]
enum HttpParserState {
    NotStarted,
    ParsingFields(HttpRequest),
    ParsingBody(HttpRequest, usize),
    Done(HttpRequest),
}

pub enum Future<T> {
    Done(T),
    Fail(&'static str),
    Wait,
}

pub struct AsyncHttpParser {
    state: HttpParserState,
    unparsed: Vec<u8>,
    reader: BufReader<TcpStream>,
}

fn parse_start(line: String) -> Result<HttpParserState, &'static str> {
    let mut words = line.split(' ');
    let method = match words.next() {
        Some("GET") => Method::Get,
        Some("POST") => Method::Post,
        Some(_) => return Err("Unknown method"),
        None => return Err("Method missing"),
    };
    let path = match words.next() {
        Some(s) => s.to_string(),
        None => return Err("Path missing"),
    };
    let version = match words.next() {
        Some(s) => s.to_string(),
        None => return Err("Version missing"),
    };

    Ok(HttpParserState::ParsingFields(HttpRequest {
        method: method,
        path: path,
        version: version,
        fields: Vec::new(),
        body: None,
    }))
}

impl AsyncHttpParser {
    pub fn as_fd(&self) -> BorrowedFd {
        return self.reader.get_ref().as_fd();
    }

    pub fn get_stream(&mut self) -> &mut TcpStream {
        self.reader.get_mut()
    }

    pub fn new(reader: BufReader<TcpStream>) -> AsyncHttpParser {
        AsyncHttpParser {
            state: HttpParserState::NotStarted,
            unparsed: Vec::new(),
            reader: reader,
        }
    }

    pub fn parse(&mut self) -> Future<HttpRequest> {
        use HttpParserState::*;
        loop {
            self.state = match &self.state {
                Done(r) => return Future::Done(r.clone()),
                NotStarted => {
                    let mut first_line = String::new();
                    if let Err(e) = self.reader.read_line(&mut first_line) {
                        if let ErrorKind::WouldBlock = e.kind() {
                            return Future::Wait;
                        }
                        warn!("Error reading line: {}", e.kind());
                        return Future::Fail("Empty request");
                    }
                    match parse_start(first_line) {
                        Ok(state) => state,
                        Err(e) => {
                            warn!("{}", e);
                            return Future::Fail(e);
                        }
                    }
                }
                ParsingFields(request) => {
                    let mut request = request.clone();
                    let mut line = String::new();
                    if let Err(e) = self.reader.read_line(&mut line) {
                        if let ErrorKind::WouldBlock = e.kind() {
                            return Future::Wait;
                        }
                        warn!("Error reading line: {}", e.kind());
                        return Future::Fail("Empty request");
                    }
                    // ignore \r\n
                    let _ = line.pop();
                    let _ = line.pop();
                    if line.is_empty() {
                        fields_end_state(request)
                    } else {
                        match parse_field(line) {
                            Some(f) => {
                                request.fields.push(f);
                                ParsingFields(request)
                            }
                            None => ParsingFields(request),
                        }
                    }
                }
                ParsingBody(request, length) => {
                    let mut request = request.clone();
                    let mut http_body = Vec::new();
                    http_body.reserve(*length);
                    unsafe {
                        http_body.set_len(*length);
                    }
                    if let Err(e) = self.reader.read_exact(&mut http_body) {
                        if let ErrorKind::WouldBlock = e.kind() {
                            return Future::Wait;
                        }
                        warn!("{}", e);
                        return Future::Fail("Error reading body");
                    }
                    info!(
                        "Read body with expected size of {} bytes\nand size of {}",
                        length,
                        http_body.len()
                    );
                    info!("Read the following: {:?}", http_body);
                    request.body = Some(http_body);
                    Done(request)
                }
            }
        }
    }
}

fn fields_end_state(request: HttpRequest) -> HttpParserState {
    use HttpParserState::*;
    match expected_content_length(&request) {
        Some(n) => ParsingBody(request, n),
        None => Done(request),
    }
}

fn expected_content_length(request: &HttpRequest) -> Option<usize> {
    let fields = &request.fields;
    if request.method != Method::Post {
        return None;
    }
    let mut exp_len = 0;
    for f in fields {
        if f.0 == "Content-Length" {
            exp_len = match f.1.parse::<usize>() {
                Ok(n) => n,
                Err(_) => {
                    warn!("Unexpected Content-Length");
                    0
                }
            };
            break;
        }
    }
    match exp_len {
        0 => None,
        n => Some(n),
    }
}
