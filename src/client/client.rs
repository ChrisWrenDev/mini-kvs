use crate::Result;
use std::net::{SocketAddr, TcpStream};
use tracing::{error, info};

pub struct KvsClient {}

impl KvsClient {
    pub fn connect(addr: SocketAddr) -> Result<()> {
        let stream = TcpStream::connect(addr);

        match stream {
            Ok(_) => info!("Server connection"),
            Err(e) => error!("Connection failed: {}", e),
        }

        Ok(())
    }
}
