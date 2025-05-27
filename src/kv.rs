#![deny(missing_docs)]
//! In Memory key/value store.

use std::collections::HashMap;

enum Status {
    Active,
    Archived,
}

struct Segment {
    id: usize,
    path: String,
}

impl Segment {
    fn new() {
        // Constructor
    }
    fn read() {
        // Get key-value at a given offset (provided by index)
    }
    fn append() {
        // Add key-value and return offset for index
    }
    fn size() {
        // Measure if compaction is needed
    }
}

struct Log {
    segments: Vec<Segment>,
    active: Segment,
    size: int64,
    baseDir: String,
    offset: int64,
}

impl Log {
    fn read() {
        // Get key-value from segment
    }
    fn append() {
        // Add key-value to acitve segment and return offset for index
    }
    fn interator() {
        // Find relevant segment
    }
    fn rollover() {
        // Handle creation of new segments
        // When are they needed?
    }
    fn compact() {
        // Remove old entries
    }
}

struct Index {
    map: HashMap<String, String>,
}

impl Index {
    fn read() {
        // Get log pointer
    }
    fn write() {
        // Add or Update key
    }
    fn remove() {
        // Remove key from index
    }
    fn build() {
        // Read log and build index
    }
}

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

        // Create set value
        // Serialize value to string
        // Append to log
        // If successful exit silently
        // If failed print error / return non-zero code
    }
    /// Get a value from store using key
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()

        // Read log to build index (key + log pointer)
        // check index for key
        // If succcessful deserialise and print value
        // If failed print "Key not found"
        // exit code 0
    }
    /// Remove key/value pair from store
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);

        // Read log to build index (key + log pointer)
        // check index for key
        // If fail print "Key not found"
        // If successful
        // -- create rm value
        // -- serialize value to string
        // -- append to log
        // -- if successful exit silently / exit code 0
        // -- if failure print error / return non-zero code
    }
}
