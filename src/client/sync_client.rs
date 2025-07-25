use crate::{ClientTraitSync, Protocol, Request, Response, Result};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use tracing::info;

pub struct TsaClientSync {
    stream: TcpStream,
}

impl TsaClientSync {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;

        info!("Server connection");
        Ok(Self { stream })
    }
}

impl ClientTraitSync for TsaClientSync {
    fn send(&mut self, request: Request) -> Result<Response> {
        let protocol = Protocol::build();
        let encoded = protocol.encode_request(&request);

        self.stream.write_all(&encoded)?;
        self.stream.flush()?;

        let mut buf = vec![0u8; 1024]; // Allocate a buffer to read into
        let n = self.stream.read(&mut buf)?;
        buf.truncate(n); // Keep only the actual bytes read

        let response = protocol.decode_response(&buf)?;

        Ok(response)
    }
}
