mod parser;
pub mod server;
pub mod socket;
pub mod types;

#[derive(Debug)]
pub struct ServerConfig {
    pub address: String,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}
