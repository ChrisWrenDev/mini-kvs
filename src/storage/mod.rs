use crate::{Result, TsaError};
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use tracing::error;

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

impl FromStr for Engine {
    type Err = TsaError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "kvs" => Ok(Engine::Kvs),
            "sled" => Ok(Engine::Sled),
            "memory" => Ok(Engine::Memory),
            _ => Err(TsaError::Protocol(format!("Invalid engine: {}", s))),
        }
    }
}

pub trait StoreTrait: Clone + Send + 'static {
    /// get the value of the given string key
    fn get(&self, key: String) -> Result<Option<String>>;

    /// set the value of the string key
    fn set(&self, key: String, val: String) -> Result<()>;

    /// remove the value of the key
    fn remove(&self, key: String) -> Result<()>;
}

#[derive(Clone)]
pub enum Storage {
    Kvs(KvStore),
    Sled(KvSled),
    Memory(KvMemory),
}

impl Storage {
    pub fn build(dir_path: PathBuf, engine: Engine) -> Result<Storage> {
        // let _config = Config::from_file("config/config.toml")?;

        check_engine(&dir_path, &engine)?;

        let store: Storage = match engine {
            Engine::Kvs => Storage::Kvs(KvStore::open(dir_path)?),
            Engine::Sled => Storage::Sled(KvSled::new(sled::open(&dir_path)?)),
            Engine::Memory => Storage::Memory(KvMemory::new()),
        };

        Ok(store)
    }
}

impl StoreTrait for Storage {
    fn get(&self, key: String) -> Result<Option<String>> {
        match self {
            Storage::Kvs(store) => store.get(key),
            Storage::Sled(store) => store.get(key),
            Storage::Memory(store) => store.get(key),
        }
    }

    fn set(&self, key: String, val: String) -> Result<()> {
        match self {
            Storage::Kvs(store) => store.set(key, val),
            Storage::Sled(store) => store.set(key, val),
            Storage::Memory(store) => store.set(key, val),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        match self {
            Storage::Kvs(store) => store.remove(key),
            Storage::Sled(store) => store.remove(key),
            Storage::Memory(store) => store.remove(key),
        }
    }
}

fn check_engine(dir_path: &PathBuf, engine: &Engine) -> Result<()> {
    let engine_str = engine.to_string();
    let engine_file = dir_path.join("engine");

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
