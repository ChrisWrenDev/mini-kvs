use crate::{ClientTrait, Protocol, Request, Result};
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
    fn send(&mut self, request: Request) -> Result<()> {
        let protocol = Protocol::build();
        let encoded = protocol.encode_request(&request);

        self.stream.write_all(&encoded)?;
        self.stream.flush()?;

        let mut buf = vec![0u8; 1024]; // Allocate a buffer to read into
        let n = self.stream.read(&mut buf)?;
        buf.truncate(n); // Keep only the actual bytes read

        info!("Client received {} bytes", n);

        let response = protocol.decode_response(&buf)?;
        debug!("Decoded response: {:?}", response);

        Ok(())
    }
}
