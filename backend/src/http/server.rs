use crate::http::parser::*;
use crate::http::types::{HttpRequest, HttpResponse};
use crate::{config::Config, info, warn};
use nix::poll::PollTimeout;
use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags};
use std::io::{BufReader, Error};
use std::net::TcpListener;
use std::os::fd::AsFd;

const DATA: u64 = 17;

type HttpHandlerT<'a> = Box<dyn HttpHandler + 'a>;
pub trait HttpHandler {
    fn handle(&self, request: HttpRequest) -> HttpResponse;
}

pub struct HttpServer<'a> {
    listener: TcpListener,
    default_handler: HttpHandlerT<'a>,
    config: &'a Config,
}

impl<'a> HttpServer<'a> {
    pub fn new(
        config: &'a Config,
        default_handler: HttpHandlerT<'a>,
    ) -> Result<HttpServer<'a>, Error> {
        let listener = match TcpListener::bind(&config.address) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        Ok(HttpServer {
            listener: listener,
            default_handler: default_handler,
            config: config,
        })
    }

    /*
    pub fn listen(&self) {
        self.listen_async();
        return;
        for stream in self.listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid stream: {}", e);
                    continue;
                }
            };
            info!(
                "Connection established with {}",
                stream.peer_addr().unwrap()
            );
            let buf_reader = BufReader::new(&mut stream);
            let http_request = match parse_http(buf_reader) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Invalid HTTP: {}", e);
                    continue;
                }
            };

            let http_response = self.default_handler.handle(http_request);
            if let Err(e) = http_response.respond(&mut stream) {
                warn!("Error responding: {}", e);
                continue;
            }
        }
    }
    */

    // listens async
    pub fn listen(&self) {
        if let Err(e) = self.listener.set_nonblocking(true) {
            warn!("{}", e);
            return;
        }
        let epoll = match Epoll::new(EpollCreateFlags::empty()) {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not make epoll object: {}", e);
                return;
            }
        };

        if let Err(e) = epoll.add(
            &self.listener.as_fd(),
            EpollEvent::new(EpollFlags::EPOLLIN, DATA),
        ) {
            warn!("Could not wait for TCP Listener: {}", e);
            return;
        }

        let mut active_parsers: Vec<AsyncHttpParser> = Vec::new();
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("Connection established with {}", addr);
                    if let Err(e) = stream.set_nonblocking(true) {
                        warn!("Stream will block: {}", e);
                    }
                    if let Err(e) =
                        epoll.add(&stream.as_fd(), EpollEvent::new(EpollFlags::EPOLLIN, DATA))
                    {
                        warn!("failed to add TCP stream to Epoll: {}", e);
                        continue;
                    }
                    let buf_reader = BufReader::new(stream);
                    active_parsers.push(AsyncHttpParser::new(buf_reader));
                    info!("{} active connections", active_parsers.len());
                }
                Err(_) => {}
            }
            for i in 0..active_parsers.len() {
                let parser = &mut active_parsers[i];
                match parser.parse() {
                    Future::Done(http_request) => {
                        let http_response = self.default_handler.handle(http_request);
                        if let Err(e) = http_response.respond(parser.get_stream()) {
                            warn!("Error responding: {}", e);
                            continue;
                        }
                        if let Err(e) = epoll.delete(&parser.as_fd()) {
                            warn!("Could not delete fd from epoll: {}", e);
                        }
                        active_parsers.remove(i);
                        break;
                    }
                    Future::Fail(e) => {
                        warn!("Invalid HTTP: {}", e);
                        active_parsers.remove(i);
                        break;
                    }
                    Future::Wait => {}
                }
            }
            let mut events = [EpollEvent::empty()];
            if let Err(e) = epoll.wait(&mut events, PollTimeout::NONE) {
                warn!("failed to wait for events: {}", e);
            }
            println!("events: {:#?}", events);
        }
    }
}
