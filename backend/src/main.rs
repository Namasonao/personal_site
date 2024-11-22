mod nodb;
mod note_db;
use log::{info, warn};
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use std::net::TcpListener;

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

fn main() {
    if let Err(e) = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info)) {
        println!("NO LOGGER");
    };
    println!("Hello");
    let listener = match TcpListener::bind("127.0.0.1:7878") {
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
    }
}
