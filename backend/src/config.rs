use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Debug)]
pub struct ParseError;
impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse config")
    }
}

#[derive(Debug)]
pub struct NetworkConfig {
    pub address: String,
    pub frontend_dir: String,
}

fn get_string(v: &serde_json::Value) -> Result<String, ParseError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(ParseError),
    }
}

pub fn parse_config_file(fp: String) -> Result<NetworkConfig, ParseError> {
    let result = fs::read(fp.clone());
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            panic!("fs::read({}) - {}", fp, e);
        }
    };

    let cfg: serde_json::Value = match serde_json::from_slice(&contents) {
        Ok(c) => c,
        Err(e) => {
            panic!("Error parsing config: {}", e);
        }
    };

    return Ok(NetworkConfig {
        address: get_string(&cfg["address"])?,
        frontend_dir: get_string(&cfg["frontend_dir"])?,
    });
}
