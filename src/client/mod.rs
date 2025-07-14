use crate::{Request, Result};
use std::net::SocketAddr;

mod sync_client;

pub trait ClientTrait {
    fn send(&mut self, request: Request) -> Result<()>;
}

pub struct Client;

impl Client {
    pub fn connect(addr: SocketAddr) -> Result<Box<dyn ClientTrait>> {
        //  let config = Config::from_file("../config/config.toml")?;
        //  match config.client {
        //      ClientConfig::Sync => {
        //          let client = sync_client::KvsClient::connect(addr)?;
        //          Ok(Box::new(client))
        //      }
        //  }

        let client = sync_client::KvsClient::connect(addr)?;
        Ok(Box::new(client))
    }
}
