use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};
use crate::warn;
use std::os::fd::{AsFd, BorrowedFd};

pub struct MyListener {
    tcp: TcpListener,
}

pub struct MyStream {
    tcp: TcpStream,
}

type Fd<'a> = BorrowedFd<'a>;
impl MyListener {
    pub fn bind(addr: &String) -> std::io::Result<MyListener> {
        let listener = TcpListener::bind(addr)?;
        if let Err(e) = listener.set_nonblocking(true) {
            warn!("error setting listener to nonblocking: {}", e);
        }
        Ok(MyListener { tcp: listener })
    }

    pub fn accept(&self) -> std::io::Result<(MyStream, SocketAddr)> {
        let (s, addr) = self.tcp.accept()?;
        if let Err(e) = s.set_nonblocking(true) {
            warn!("error setting stream to nonblocking: {}", e);
        }
        Ok((MyStream { tcp: s }, addr))
    }
}

impl AsFd for MyListener {
    fn as_fd(&self) -> Fd<'_> {
        self.tcp.as_fd()
    }

}

impl MyStream {
    pub fn new(tcp: TcpStream) -> MyStream {
        if let Err(e) = tcp.set_nonblocking(true) {
            warn!("error setting TCP stream to nonblocking: {}", e);
        }
        MyStream { tcp }
    }
}

impl AsFd for MyStream {
    fn as_fd(&self) -> Fd<'_> {
        self.tcp.as_fd()
    }
}

impl Read for MyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tcp.read(buf)
    }
}

impl Write for MyStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tcp.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tcp.flush()
    }
}
