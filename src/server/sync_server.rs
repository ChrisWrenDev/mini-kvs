use crate::{
    Engine, KvsError, PoolType, Protocol, Request, Response, Result, ServerTrait, Storage,
    StoreTrait, ThreadPool,
};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

pub struct SyncServer {
    pub addr: SocketAddr,
    pub store: Arc<Mutex<Storage>>,
    pub pool: ThreadPool,
    pub shutdown_tx: Sender<()>,
    pub shutdown_rx: Receiver<()>,
}

impl SyncServer {
    pub fn new(
        addr: SocketAddr,
        engine: Engine,
        pool: PoolType,
        num_threads: u32,
        dir_path: PathBuf,
    ) -> Result<SyncServer> {
        let store = Arc::new(Mutex::new(Storage::build(dir_path, engine)?));

        let pool = ThreadPool::run(pool, num_threads)?;

        let (shutdown_tx, shutdown_rx) = channel::<()>();

        Ok(SyncServer {
            addr,
            store,
            pool,
            shutdown_tx,
            shutdown_rx,
        })
    }
}
impl ServerTrait for SyncServer {
    fn run(&mut self) -> Result<()> {
        info!("Server starting at {}", self.addr);

        let listener = TcpListener::bind(self.addr)?;

        for stream in listener.incoming() {
            if self.shutdown_rx.try_recv().is_ok() {
                println!("Shutting down server");
                break;
            }
            let stream = stream?;
            let store = Arc::clone(&self.store);
            self.pool.spawn(move || {
                if let Err(e) = handle_connecton(stream, store) {
                    error!("Failed to handle connection: {}", e);
                }
            });
        }

        println!("Shutting down.");

        Ok(())
    }
    fn shutdown(&self) -> Sender<()> {
        self.shutdown_tx.clone()
    }
}
fn handle_connecton(mut stream: TcpStream, store: Arc<Mutex<Storage>>) -> Result<()> {
    let mut buffer = vec![0u8; 1024]; // Allocate 1 KB
    let bytes_read = stream.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let protocol = Protocol::build();
    let request = protocol.decode_request(&buffer)?;

    let store = store.lock().map_err(|_| KvsError::LockPoisoned)?;
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
