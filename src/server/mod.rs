use crate::{Engine, Result};
use std::net::SocketAddr;

mod sync_server;

pub trait ServerTrait {
    fn run(&mut self) -> Result<()>;
}

pub struct Server;

impl Server {
    pub fn build(addr: SocketAddr, engine: Engine) -> Result<Box<dyn ServerTrait>> {
        // let config = Config::from_file("../config/config.toml")?;

        // match config.server {
        //     ServerConfig::Sync => Ok(Box::new(sync_server::SyncServer::new(addr, engine)?)),
        // }

        Ok(Box::new(sync_server::SyncServer::new(addr, engine)?))
    }
}
