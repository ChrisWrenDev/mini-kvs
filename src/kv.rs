#![deny(missing_docs)]
//! In Memory key/value store.

use std::collections::HashMap;

/// KvStore creates HashMap of key/value pairs
///
/// # Examples
///
/// ```
/// let store = KvStore::new();
/// store.set("fruit".to_owned(), "apple".to_owned());
/// let fruit = store.get("fruit".to_owned());
///
/// assert_eq!(fruit, Some("apple".to_owned()));
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// Create a key/value store
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }
    /// Add a key/value pair to store
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }
    /// Get a value from store using key
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }
    /// Remove key/value pair from store
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
