use crate::{KvStore, Result};
use std::net::{SocketAddr, TcpListener};
use tracing::{error, info};

pub struct KvsServer {
    addr: SocketAddr,
    engine: KvStore,
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: KvStore) -> KvsServer {
        KvsServer { addr, engine }
    }
    pub fn run(self) -> Result<()> {
        info!("Server starting at {}", self.addr);
        let listener = TcpListener::bind(self.addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(_) => info!("Server connection"),
                Err(e) => error!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}
