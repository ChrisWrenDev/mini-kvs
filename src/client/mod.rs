use crate::{Request, Response, Result};
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;

mod sync_client;
pub use sync_client::TsaClientSync;

mod async_client;
pub use async_client::TsaClientAsync;

pub trait ClientTraitSync {
    fn send(&mut self, request: Request) -> Result<Response>;
}

pub trait ClientTraitAsync {
    async fn send(&mut self, request: Request) -> Result<Response>;
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ClientType {
    Sync,
    Async,
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ClientType::Sync => "sync",
            ClientType::Async => "async",
        };
        write!(f, "{}", s)
    }
}

pub enum ClientSync {
    Sync(TsaClientSync),
}

pub enum ClientAsync {
    Async(TsaClientAsync),
}

impl ClientSync {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        //  let config = Config::from_file("../config/config.toml")?;
        //  match config.client {
        //      ClientConfig::Sync => {
        //          let client = sync_client::TsaClient::connect(addr)?;
        //          Ok(Box::new(client))
        //      }
        //  }

        let client = sync_client::TsaClientSync::connect(addr)?;
        Ok(ClientSync::Sync(client))
    }
}

impl ClientTraitSync for ClientSync {
    fn send(&mut self, request: Request) -> Result<Response> {
        match self {
            ClientSync::Sync(client) => client.send(request),
        }
    }
}

impl ClientAsync {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let client = async_client::TsaClientAsync::connect(addr).await?;
        Ok(ClientAsync::Async(client))
    }
}
impl ClientTraitAsync for ClientAsync {
    async fn send(&mut self, request: Request) -> Result<Response> {
        match self {
            ClientAsync::Async(client) => client.send(request).await,
        }
    }
}
