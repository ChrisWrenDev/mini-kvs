use crate::{ClientTraitAsync, Protocol, Request, Response, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::info;

pub struct KvsClientAsync {
    stream: TcpStream,
}

impl KvsClientAsync {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        info!("Server connection");
        Ok(Self { stream })
    }
}

impl ClientTraitAsync for KvsClientAsync {
    async fn send(&mut self, request: Request) -> Result<Response> {
        let protocol = Protocol::build();
        let encoded = protocol.encode_request(&request);

        self.stream.write_all(&encoded).await?;
        self.stream.flush().await?;

        let mut buf = vec![0u8; 1024]; // Allocate a buffer to read into
        let n = self.stream.read(&mut buf).await?;
        buf.truncate(n); // Keep only the actual bytes read

        let response = protocol.decode_response(&buf)?;

        Ok(response)
    }
}
