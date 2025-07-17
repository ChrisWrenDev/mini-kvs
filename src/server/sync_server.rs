use crate::{Engine, Protocol, Request, Response, Result, ServerTrait, Storage, StoreTrait};
use std::env::current_dir;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use tracing::{error, info};

pub struct SyncServer {
    pub addr: SocketAddr,
    pub store: Storage,
}

impl SyncServer {
    pub fn new(addr: SocketAddr, engine: Engine) -> Result<SyncServer> {
        let dir_path = current_dir()?;
        let store = Storage::build(dir_path, engine)?;

        Ok(SyncServer { addr, store })
    }
}
impl ServerTrait for SyncServer {
    fn run(&mut self) -> Result<()> {
        info!("Server starting at {}", self.addr);

        let listener = TcpListener::bind(self.addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connecton(stream, &mut self.store)?;
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}
fn handle_connecton(mut stream: TcpStream, store: &mut Storage) -> Result<()> {
    let mut buffer = vec![0u8; 1024]; // Allocate 1 KB
    let bytes_read = stream.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let protocol = Protocol::build();
    let request = protocol.decode_request(&buffer)?;

    let response: Response = match request {
        Request::Set { key, value } => {
            store.set(key, value)?;
            Response::Ok
        }
        Request::Get { key } => match store.get(key)? {
            Some(value) => {
                info!("Get Value: {}", value);
                Response::Value(value)
            }
            None => Response::NotFound,
        },
        Request::Remove { key } => match store.remove(key) {
            Ok(_) => Response::Ok,
            Err(_) => Response::NotFound,
        },
    };

    let encoded = protocol.encode_response(&response);

    info!("Encoded Server: {:?}", encoded);

    stream.write_all(&encoded)?;
    stream.flush()?;

    Ok(())
}
