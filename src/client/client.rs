use crate::{Config, Protocol, Request, Result, client::ClientTrait};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use tracing::{debug, info};

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: SocketAddr) -> Result<KvsClient> {
        let stream = TcpStream::connect(addr)?;

        info!("Server connection");
        Ok(KvsClient { stream })
    }
}

impl ClientTrait for KvsClient {
    fn send(&mut self, config: &Config, request: Request) -> Result<()> {
        let protcol = Protocol::build(config);
        let encoded = protcol.encode_request(&request);

        self.stream.write_all(&encoded)?;
        self.stream.flush()?;

        let mut buf = Vec::new();

        let n = self.stream.read(&mut buf)?;
        let response = protcol.decode_response(&buf[..n])?;

        debug!("{:?}", response);

        Ok(())
    }
}
