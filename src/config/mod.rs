use crate::common::Result;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub protocol: ProtocolConfig,
    pub serialization: SerializationConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageConfig {
    Kvs,
    Sled,
    Memory,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerConfig {
    Sync,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientConfig {
    Sync,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolConfig {
    Result,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SerializationConfig {
    Binary,
}
