use crate::common::Result;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    storage: Option<String>,
    server: Option<String>,
    protocol: Option<String>,
    serialization: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
