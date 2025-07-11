use crate::{Result, ServerTrait, StoreTrait};
use std::io::{BufReader, BufWriter};
use std::net::{SocketAddr, TcpListener, TcpStream};
use tracing::{error, info};

pub struct KvsServer {
    pub addr: SocketAddr,
    pub engine: Box<dyn StoreTrait>,
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: Box<dyn StoreTrait>) -> KvsServer {
        KvsServer { addr, engine }
    }
}
impl ServerTrait for KvsServer {
    fn run(&self) -> Result<()> {
        info!("Server starting at {}", self.addr);
        let listener = TcpListener::bind(self.addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connecton(stream);
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}

fn handle_connecton(stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    Ok(())
}
