use crate::{Engine, PoolType, Result};
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

mod sync_server;
pub use sync_server::SyncServer;

pub trait ServerTrait {
    fn run(&mut self) -> Result<()>;
    fn shutdown(&self) -> Sender<()>;
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ServerType {
    Sync,
}

impl Display for ServerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ServerType::Sync => "sync",
        };
        write!(f, "{}", s)
    }
}

pub enum Server {
    Sync(SyncServer),
}

impl Server {
    pub fn build(
        addr: SocketAddr,
        engine: Engine,
        pool: PoolType,
        threads: u32,
        dir_path: PathBuf,
    ) -> Result<Server> {
        // let config = Config::from_file("../config/config.toml")?;

        // match config.server {
        //     ServerConfig::Sync => Ok(Box::new(sync_server::SyncServer::new(addr, engine)?)),
        // }

        let server = SyncServer::new(addr, engine, pool, threads, dir_path)?;

        Ok(Server::Sync(server))
    }
}

impl ServerTrait for Server {
    fn run(&mut self) -> Result<()> {
        match self {
            Server::Sync(server) => server.run(),
        }
    }
    fn shutdown(&self) -> Sender<()> {
        match self {
            Server::Sync(server) => server.shutdown(),
        }
    }
}
