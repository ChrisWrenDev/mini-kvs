use crate::{Result, ServerTrait, StoreTrait};
use std::net::{SocketAddr, TcpListener};
use tracing::{error, info};

pub struct KvsServer<E: StoreTrait> {
    pub addr: SocketAddr,
    pub engine: E,
}

impl<E: StoreTrait> KvsServer<E> {
    pub fn new(addr: SocketAddr, engine: E) -> KvsServer<E> {
        KvsServer { addr, engine }
    }
}
impl<E: StoreTrait> ServerTrait for KvsServer<E> {
    fn run(&self) -> Result<()> {
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
