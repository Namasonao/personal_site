use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::fs;
use http::{ServerConfig, TlsConfig};

#[derive(Debug)]
pub enum ParseError {
    Missing(String),
    MissingTls(String),
    Syntax(Box<dyn Error>),
}
impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;
        match self {
            Missing(s) => write!(f, "missing `{}` in json config", s),
            MissingTls(s) => write!(
                f,
                "missing `{}` for tls configuration (or add `allow_insecure` = true)",
                s
            ),
            Syntax(e) => write!(f, "syntax error: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub frontend_dir: String,
    pub database: String,
    pub http: ServerConfig,
}

fn get_string(cfg: &serde_json::Value, name: &str) -> Result<String, ParseError> {
    let v = &cfg[name];
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(ParseError::Missing(name.to_string())),
    }
}

fn get_tls(cfg: &serde_json::Value) -> Result<TlsConfig, ParseError> {
    use ParseError::*;
    let cert = match get_string(&cfg, "cert") {
        Err(Missing(s)) => Err(MissingTls(s)),
        a => a,
    }?;
    let key = match get_string(&cfg, "key") {
        Err(Missing(s)) => Err(MissingTls(s)),
        a => a,
    }?;
    Ok(TlsConfig { cert, key })
}

pub fn parse_config_file(fp: &String) -> Result<Config, ParseError> {
    let result = fs::read(fp);
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            panic!("fs::read({}) - {}", fp, e);
        }
    };

    let cfg: serde_json::Value = match serde_json::from_slice(&contents) {
        Ok(c) => c,
        Err(e) => {
            return Err(ParseError::Syntax(e.into()));
        }
    };

    let allow_insecure = match &cfg["allow_insecure"] {
        serde_json::Value::Bool(true) => true,
        _ => false,
    };
    let tls = if allow_insecure {
        get_tls(&cfg).ok()
    } else {
        Some(get_tls(&cfg)?)
    };
    let http = ServerConfig {
        address: get_string(&cfg, "address")?,
        tls,
    };
    return Ok(Config {
        frontend_dir: get_string(&cfg, "frontend_dir")?,
        database: get_string(&cfg, "database")?,
        http,
    });
}
