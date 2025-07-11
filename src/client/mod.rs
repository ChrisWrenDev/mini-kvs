use crate::{Config, Request, Result, config::ClientConfig};
use std::net::SocketAddr;

mod client;

pub trait ClientTrait {
    fn send(&mut self, config: &Config, request: Request) -> Result<()>;
}

pub struct Client;

impl Client {
    pub fn connect(config: &Config, addr: SocketAddr) -> Result<Box<dyn ClientTrait>> {
        match config.client {
            ClientConfig::Sync => {
                let client = client::KvsClient::connect(addr)?;
                Ok(Box::new(client))
            }
        }
    }
}
