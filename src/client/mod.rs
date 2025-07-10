use crate::{Config, Result};
use std::net::SocketAddr;

mod client;

pub trait ClientTrait {
    fn connect(&self, addr: SocketAddr) -> Result<()>;
}

pub enum Client {
    Kvs(client::KvsClient),
}

impl Client {
    pub fn build(config: &Config) -> Self {
        return Client::Kvs(client::KvsClient);
    }
}
