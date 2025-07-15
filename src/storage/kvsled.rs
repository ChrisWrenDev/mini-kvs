use super::StoreTrait;
use crate::{KvsError, Result};
use sled::{Db, Tree};

/// Wrapper of `sled::Db`
#[derive(Clone)]
pub struct KvSled(Db);

impl KvSled {
    /// Creates a `KvSled` from `sled::Db`.
    pub fn new(db: Db) -> Self {
        KvSled(db)
    }
}

impl StoreTrait for KvSled {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let tree: &Tree = &self.0;
        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let tree: &Tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let tree: &Tree = &self.0;
        tree.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
