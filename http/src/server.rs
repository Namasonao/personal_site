use crate::ServerConfig;
use crate::parser::*;
use crate::socket::Listener;
use crate::types::{HttpRequest, HttpResponse, Responder};
use log::{info, warn};
use nix::poll::PollTimeout;
use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags};
use std::io::{BufReader, Error};
use std::os::fd::AsFd;
use std::time::Duration;

const DATA: u64 = 17;

type HttpHandlerT<'a> = Box<dyn HttpHandler + 'a>;
pub trait HttpHandler {
    fn handle(&self, request: HttpRequest) -> HttpResponse;
}

pub struct HttpServer<'a> {
    listener: Listener,
    default_handler: HttpHandlerT<'a>,
}

impl<'a> HttpServer<'a> {
    pub fn new(
        config: &'a ServerConfig,
        default_handler: HttpHandlerT<'a>,
    ) -> Result<HttpServer<'a>, Error> {
        let mut listener = match Listener::bind(&config.address) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        if let Some(tls) = &config.tls {
            listener.enable_tls(&tls.cert, &tls.key).unwrap();
        }
        Ok(HttpServer {
            listener: listener,
            default_handler: default_handler,
        })
    }

    // listens async
    pub fn listen(&self) {
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
        let mut active_responders: Vec<Responder> = Vec::new();
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("Connection established with {}", addr);
                    if let Err(e) =
                        epoll.add(&stream.as_fd(), EpollEvent::new(EpollFlags::EPOLLIN, DATA))
                    {
                        warn!("failed to add TCP stream to Epoll: {}", e);
                        continue;
                    }
                    let buf_reader = BufReader::new(stream);
                    let mut parser = AsyncHttpParser::new(buf_reader);
                    parser.set_timeout(Duration::from_secs(2));
                    active_parsers.push(parser);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                    } else {
                        warn!("accept error: {}", e);
                    }
                }
            }
            let mut i = 0;
            while i < active_parsers.len() {
                let parser = &mut active_parsers[i];
                match parser.parse() {
                    Future::Done(http_request) => {
                        info!("{}", http_request);
                        let parser = active_parsers.swap_remove(i);
                        let http_response = self.default_handler.handle(http_request);
                        let mut responder =
                            Responder::from_http_response(http_response, parser.into_stream());
                        if match responder.respond() {
                            Future::Done(()) => false,
                            Future::Wait => true,
                            Future::Fail(s) => {
                                warn!("response failed: {}", s);
                                false
                            }
                        } {
                            if let Err(e) = epoll.modify(
                                responder.as_fd(),
                                &mut EpollEvent::new(EpollFlags::EPOLLOUT, DATA),
                            ) {
                                warn!("failed to modify fd: {}", e);
                            }
                            info!("could not send full response... adding responder to queue");
                            active_responders.push(responder);
                            info!("queue of {} responders", active_responders.len());
                        } else {
                            if let Err(e) = epoll.delete(responder.as_fd()) {
                                warn!("failed to delete fd: {}", e);
                            }
                        }
                    }
                    Future::Fail(e) => {
                        warn!("Invalid HTTP: {}", e);
                        let _ = active_parsers.swap_remove(i);
                    }
                    Future::Wait => {
                        i += 1;
                    }
                };
            }
            let mut i = 0;
            while i < active_responders.len() {
                let responder = &mut active_responders[i];
                match responder.respond() {
                    Future::Done(()) => {
                        if let Err(e) = epoll.delete(responder.as_fd()) {
                            warn!("failed to delete fd: {}", e);
                        }
                        active_responders.swap_remove(i);
                    }
                    Future::Fail(e) => {
                        warn!("Failed to respond: {}", e);
                        if let Err(e) = epoll.delete(responder.as_fd()) {
                            warn!("failed to delete fd: {}", e);
                        }
                        let _ = active_responders.swap_remove(i);
                    }
                    Future::Wait => {
                        i += 1;
                    }
                };
            }

            let mut events = [EpollEvent::empty()];
            if let Err(e) = epoll.wait(&mut events, PollTimeout::NONE) {
                warn!("failed to wait for events: {}", e);
            }
        }
    }
}
