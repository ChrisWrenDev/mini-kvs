use crate::{Result, StoreTrait};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvMemory;
/// let mut store = KvMemory::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
#[derive(Default, Clone, Debug)]
pub struct KvMemory {
    map: Arc<Mutex<HashMap<String, String>>>,
}

impl KvMemory {
    /// Creates a `KvStore`.
    pub fn new() -> KvMemory {
        KvMemory {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
impl StoreTrait for KvMemory {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.map.lock()?.insert(key, value);
        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.lock()?.get(&key).cloned())
    }

    /// Remove a given key.
    fn remove(&self, key: String) -> Result<()> {
        self.map.lock()?.remove(&key);
        Ok(())
    }
}
