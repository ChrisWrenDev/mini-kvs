use crate::{Config, Result, StoreTrait};
use std::net::SocketAddr;

mod server;

pub trait ServerTrait {
    fn run(&self) -> Result<()>;
}

pub struct Server;

impl Server {
    pub fn build(config: &Config, addr: SocketAddr, engine: impl StoreTrait) -> impl ServerTrait {
        server::KvsServer::new(addr, engine)
    }
}
