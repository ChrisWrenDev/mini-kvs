use crate::Result;
use clap::ValueEnum;
use std::env::current_dir;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::process::exit;
use tracing::{error, info};

// looks for file or folder with mod.rs
mod kvmemory;
mod kvsled;
mod kvstore;

pub use kvmemory::KvMemory;
pub use kvsled::KvSled;
pub use kvstore::KvStore;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Engine {
    Kvs,
    Sled,
    Memory,
}

impl Display for Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Engine::Kvs => "kvs",
            Engine::Sled => "sled",
            Engine::Memory => "memory",
        };
        write!(f, "{}", s)
    }
}

pub trait StoreTrait {
    /// get the value of the given string key
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// set the value of the string key
    fn set(&mut self, key: String, val: String) -> Result<()>;

    /// remove the value of the key
    fn remove(&mut self, key: String) -> Result<()>;
}

pub struct Storage;

impl Storage {
    pub fn build(engine: Engine) -> Result<Box<dyn StoreTrait>> {
        // let _config = Config::from_file("config/config.toml")?;

        check_engine(&engine)?;

        let dir_path = current_dir()?;

        let store: Box<dyn StoreTrait> = match engine {
            Engine::Kvs => Box::new(kvstore::KvStore::open(dir_path)?),
            Engine::Sled => Box::new(kvsled::KvSled::new(sled::open(dir_path)?)),
            Engine::Memory => Box::new(kvmemory::KvMemory::new()),
        };

        Ok(store)
    }
}

fn check_engine(engine: &Engine) -> Result<()> {
    let engine_str = engine.to_string();
    let engine_file = current_dir()?.join("engine");

    if !engine_file.exists() {
        fs::write(&engine_file, &engine_str)?;
        return Ok(());
    }

    let current_engine = fs::read_to_string(&engine_file)?.trim().to_string();

    if current_engine == engine_str {
        return Ok(());
    }

    error!(
        "Engine mismatch: existing engine is '{}', but received '{}'",
        current_engine, engine_str
    );
    exit(1);
}
