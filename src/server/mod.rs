use crate::{Config, Result, StoreTrait, config::ServerConfig};
use std::net::SocketAddr;

mod server;

pub trait ServerTrait {
    fn run(&self) -> Result<()>;
}

pub struct Server;

impl Server {
    pub fn build(
        config: &Config,
        addr: SocketAddr,
        engine: Box<dyn StoreTrait>,
    ) -> Result<Box<dyn ServerTrait>> {
        match config.server {
            ServerConfig::Sync => return Ok(Box::new(server::KvsServer::new(addr, engine))),
        }
    }
}
