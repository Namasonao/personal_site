use log::warn;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::{AsFd, BorrowedFd};
use std::sync::Arc;

pub struct Listener {
    tcp: TcpListener,
    tls_config: Option<Arc<rustls::ServerConfig>>,
}

pub struct Stream {
    tcp: TcpStream,
    conn: Option<rustls::ServerConnection>,
    //tls: rustls::Stream<'static, rustls::ServerConnection, TcpStream>,
}

fn make_tls_config(
    cert_file: &str,
    private_key_file: &str,
) -> Result<rustls::ServerConfig, rustls::Error> {
    let certs = CertificateDer::pem_file_iter(cert_file)
        .unwrap()
        .map(|cert| cert.unwrap())
        .collect();
    let private_key = PrivateKeyDer::from_pem_file(private_key_file).unwrap();
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;
    Ok(config)
}

type Fd<'a> = BorrowedFd<'a>;
impl Listener {
    pub fn bind(addr: &str) -> std::io::Result<Listener> {
        let listener = TcpListener::bind(addr)?;
        if let Err(e) = listener.set_nonblocking(true) {
            warn!("error setting listener to nonblocking: {}", e);
        }
        Ok(Listener {
            tcp: listener,
            tls_config: None,
        })
    }
    pub fn enable_tls(&mut self, cert_fp: &str, key_fp: &str) -> Result<(), rustls::Error> {
        if let Some(_) = self.tls_config {
            panic!("tls already enabled");
        }
        let config = make_tls_config(cert_fp, key_fp)?;
        self.tls_config = Some(Arc::new(config));
        Ok(())
    }

    pub fn accept(&self) -> std::io::Result<(Stream, SocketAddr)> {
        let (s, addr) = self.tcp.accept()?;
        if let Err(e) = s.set_nonblocking(true) {
            warn!("error setting stream to nonblocking: {}", e);
        }
        if let Some(config) = &self.tls_config {
            let Ok(conn) = rustls::ServerConnection::new(config.clone()) else {
                panic!("could not make tls connection")
            };
            Ok((
                Stream {
                    tcp: s,
                    conn: Some(conn),
                },
                addr,
            ))
        } else {
            warn!("insecure connection");
            Ok((Stream { tcp: s, conn: None }, addr))
        }
    }
}

impl AsFd for Listener {
    fn as_fd(&self) -> Fd<'_> {
        self.tcp.as_fd()
    }
}

impl AsFd for Stream {
    fn as_fd(&self) -> Fd<'_> {
        self.tcp.as_fd()
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(conn) = &mut self.conn {
            let mut tls_stream = rustls::Stream::new(conn, &mut self.tcp);
            tls_stream.read(buf)
        } else {
            self.tcp.read(buf)
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(conn) = &mut self.conn {
            let mut tls_stream = rustls::Stream::new(conn, &mut self.tcp);
            tls_stream.write(buf)
        } else {
            self.tcp.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(conn) = &mut self.conn {
            let mut tls_stream = rustls::Stream::new(conn, &mut self.tcp);
            tls_stream.flush()
        } else {
            self.tcp.flush()
        }
    }
}
