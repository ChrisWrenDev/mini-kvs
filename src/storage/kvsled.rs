use super::StoreTrait;
use crate::{KvsError, Result};
use sled::{Db, Tree};
use std::sync::{Arc, Mutex};

/// Wrapper of `sled::Db`
#[derive(Clone)]
pub struct KvSled {
    db: Arc<Mutex<Db>>,
}

impl KvSled {
    /// Creates a `KvSled` from `sled::Db`.
    pub fn new(db: Db) -> Self {
        KvSled {
            db: Arc::new(Mutex::new(db)),
        }
    }
}

impl StoreTrait for KvSled {
    fn set(&self, key: String, value: String) -> Result<()> {
        let tree = self.db.lock()?;
        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let tree = self.db.lock()?;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&self, key: String) -> Result<()> {
        let tree = self.db.lock()?;
        tree.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
