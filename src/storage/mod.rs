use crate::{Config, Result};
use std::path::Path;
// looks for file or folder with mod.rs
mod kvmemory;
mod kvstore;

pub use kvmemory::KvMemory;
pub use kvstore::KvStore;

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
    pub fn build(config: &Config, dir_path: &Path) -> Result<Box<dyn StoreTrait>> {
        let store = kvstore::KvStore::open(&dir_path)?;
        Ok(Box::new(store))
    }
}
