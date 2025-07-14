use crate::Result;
use clap::ValueEnum;
use std::path::Path;
use tracing::info;
// looks for file or folder with mod.rs
mod kvmemory;
mod kvstore;

pub use kvmemory::KvMemory;
pub use kvstore::KvStore;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Engine {
    Kvs,
    Sled,
    Memory,
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
    pub fn build(dir_path: &Path, _engine: Engine) -> Result<Box<dyn StoreTrait>> {
        // let _config = Config::from_file("config/config.toml")?;
        info!("Storage: {:?}", dir_path);
        let store = kvstore::KvStore::open(dir_path)?;
        Ok(Box::new(store))
    }
}
