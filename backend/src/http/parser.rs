use crate::http::types::*;
use crate::socket::MyStream;
use crate::warn;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::mem;
use std::os::fd::{AsFd, BorrowedFd};
use std::time::{Duration, SystemTime};

enum HttpParserState {
    NotStarted,
    ParsingFields(HttpRequest),
    ParsingBody(HttpRequest, usize),
    Done(HttpRequest),
    Moved,
}

pub enum Future<T> {
    Done(T),
    Fail(&'static str),
    Wait,
}

struct TimeoutInfo {
    start: SystemTime,
    duration: Duration,
}

pub struct AsyncHttpParser {
    state: HttpParserState,
    reader: BufReader<MyStream>,
    timeout_info: Option<TimeoutInfo>,
}

impl AsyncHttpParser {
    pub fn as_fd(&self) -> BorrowedFd {
        return self.reader.get_ref().as_fd();
    }

    pub fn get_stream(&mut self) -> &mut MyStream {
        self.reader.get_mut()
    }

    pub fn new(reader: BufReader<MyStream>) -> AsyncHttpParser {
        AsyncHttpParser {
            state: HttpParserState::NotStarted,
            reader: reader,
            timeout_info: None,
        }
    }
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_info = Some(TimeoutInfo {
            start: SystemTime::now(),
            duration: duration,
        });
    }

    fn timeout(&self) -> bool {
        let timeout_info = match &self.timeout_info {
            Some(t) => t,
            None => return false,
        };
        match timeout_info.start.elapsed() {
            Ok(d) => d >= timeout_info.duration,
            Err(_) => true,
        }
    }

    fn parse_start(&mut self) -> Future<()> {
        let HttpParserState::NotStarted = &self.state else {
            return Future::Fail("Unexpected State");
        };

        let mut first_line = String::new();
        if let Err(e) = self.reader.read_line(&mut first_line) {
            if let ErrorKind::WouldBlock = e.kind() {
                return Future::Wait;
            }
            warn!("Error reading line: {}", e.kind());
            return Future::Fail("Empty request");
        }
        self.state = match parse_start(first_line) {
            Ok(state) => state,
            Err(e) => {
                warn!("{}", e);
                return Future::Fail(e);
            }
        };
        Future::Done(())
    }

    fn parse_fields(&mut self) -> Future<()> {
        let HttpParserState::ParsingFields(_) = &self.state else {
            return Future::Fail("Unexpected state");
        };
        let mut line = String::new();
        if let Err(e) = self.reader.read_line(&mut line) {
            if let ErrorKind::WouldBlock = e.kind() {
                return Future::Wait;
            }
            warn!("Error reading line: {}", e.kind());
            return Future::Fail("Empty request");
        }
        // move state to avoid duplication
        let HttpParserState::ParsingFields(mut request) =
            mem::replace(&mut self.state, HttpParserState::Moved)
        else {
            warn!("If this is printed there is trouble");
            return Future::Fail("Unexpected state");
        };

        // ignore \r\n
        let _ = line.pop();
        let _ = line.pop();
        self.state = if line.is_empty() {
            fields_end_state(request)
        } else {
            match parse_field(line) {
                Some(f) => {
                    request.fields.push(f);
                    HttpParserState::ParsingFields(request)
                }
                None => HttpParserState::ParsingFields(request),
            }
        };
        Future::Done(())
    }

    fn parse_body(&mut self) -> Future<()> {
        let HttpParserState::ParsingBody(_, length) = &self.state else {
            return Future::Fail("Unexpected state");
        };
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
        let HttpParserState::ParsingBody(mut request, _) =
            mem::replace(&mut self.state, HttpParserState::Moved)
        else {
            return Future::Fail("Unexpected state");
        };
        request.body = Some(http_body);
        self.state = HttpParserState::Done(request);
        Future::Done(())
    }

    pub fn parse(&mut self) -> Future<HttpRequest> {
        if self.timeout() {
            return Future::Fail("TIMEOUT");
        }
        use HttpParserState::*;
        loop {
            //let old_state = mem::replace(&mut self.state, Moved);
            let success = match &self.state {
                Moved => {
                    warn!("Can not parse moved state");
                    return Future::Fail("??");
                }
                Done(_) => {
                    let Done(r) = mem::replace(&mut self.state, Moved) else {
                        return Future::Fail("");
                    };
                    return Future::Done(r);
                }
                NotStarted => self.parse_start(),
                ParsingFields(_) => self.parse_fields(),
                ParsingBody(_, _) => self.parse_body(),
            };
            match success {
                Future::Done(_) => {}
                Future::Wait => return Future::Wait,
                Future::Fail(e) => return Future::Fail(e),
            };
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
    let mut version = match words.next() {
        Some(s) => s.to_string(),
        None => return Err("Version missing"),
    };
    // ignore \r\n
    _ = version.pop();
    _ = version.pop();

    Ok(HttpParserState::ParsingFields(HttpRequest {
        method: method,
        path: path,
        version: version,
        fields: Vec::new(),
        body: None,
    }))
}

fn parse_field(line: String) -> Option<Field> {
    let mut it = line.split(':');
    let key = it.next()?.to_string();
    let mut val = it.next()?.to_string();
    val.remove(0);
    Some((key, val))
}
