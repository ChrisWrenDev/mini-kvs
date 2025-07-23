use crate::{Request, Response, Result};
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;

mod sync_client;
pub use sync_client::KvsClient;

pub trait ClientTrait {
    fn send(&mut self, request: Request) -> Result<Response>;
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ClientType {
    Sync,
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ClientType::Sync => "sync",
        };
        write!(f, "{}", s)
    }
}

pub enum Client {
    Sync(KvsClient),
}

impl Client {
    pub fn connect(addr: SocketAddr) -> Result<Client> {
        //  let config = Config::from_file("../config/config.toml")?;
        //  match config.client {
        //      ClientConfig::Sync => {
        //          let client = sync_client::KvsClient::connect(addr)?;
        //          Ok(Box::new(client))
        //      }
        //  }

        let client = sync_client::KvsClient::connect(addr)?;
        Ok(Client::Sync(client))
    }
}

impl ClientTrait for Client {
    fn send(&mut self, request: Request) -> Result<Response> {
        match self {
            Client::Sync(client) => client.send(request),
        }
    }
}
