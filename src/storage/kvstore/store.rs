#![deny(missing_docs)]
//! In Memory key/value store.
use super::entry::Entry;
use super::segment::{SegmentReader, SegmentWriter};
use crate::{Result, StoreTrait, TsaError};
use dashmap::DashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

const MAX_LOG_FILE_SIZE: u64 = 4 * 1024 * 1024; // 4 MB
// const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1 MB
const COMPACTION_THRESHOLD: u64 = 1024; // 1 MB

#[derive(Debug, Clone)]
pub struct CommandPos {
    pub file_id: u64, // Which file
    pub offset: u64,  // Where in the file
    pub length: u64,  // Number of bytes
}

/// KvStore creates HashMap of key/value pairs
///
/// # Example
///
/// ```
/// use tempfile::TempDir;
/// use kvs::{KvStore, Result};
///
/// fn main() -> Result<()> {
///     let temp_dir = TempDir::new().expect("unable to create temporary working directory");
///     let mut store = KvStore::open(temp_dir.path())?;
///
///     store.set("language".to_string(), "Rust".to_string())?;
///     assert_eq!(store.get("language".to_string())?, Some("Rust".to_string()));
///
///     store.remove("language".to_string())?;
///     assert_eq!(store.get("language".to_string())?, None);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct KvStore {
    base_dir: PathBuf,
    readers: Arc<DashMap<String, SegmentReader>>,
    writer: Arc<Mutex<SegmentWriter>>,
    size: Arc<AtomicU64>,
    index: Arc<DashMap<String, CommandPos>>,
    stale_entries: Arc<AtomicU64>,
    compaction: Arc<AtomicBool>,
}

impl KvStore {
    /// Create a key/value store
    pub fn open(dir_path: PathBuf) -> Result<KvStore> {
        // Add segments to vector
        let readers = DashMap::new();

        // Create index
        let mut index = DashMap::new();

        // Get all files
        // Check directory for log files
        let mut file_ids = fs::read_dir(&dir_path)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();

                if path.extension()?.to_str()? != "log" {
                    return None;
                }

                let stem_str = path.file_stem()?.to_str()?;
                stem_str.parse::<u64>().ok()
            })
            .collect::<Vec<u64>>();

        file_ids.sort_unstable();

        // Create segment for each log file
        let active = file_ids.iter().copied().max().unwrap_or(1);

        // Calculate size of log
        let mut size = 0;

        // Calculate stale entries in log
        let mut stale_entries = 0;

        let writer = match !file_ids.is_empty() {
            true => SegmentWriter::open(&dir_path, active)?,
            false => SegmentWriter::new(&dir_path, active)?,
        };

        // If no files, create one
        if file_ids.is_empty() {
            file_ids.push(active);
        }

        // loop newest to oldest (highest to lowest)
        for id in file_ids {
            // Create segment for file
            let mut reader = SegmentReader::open(&dir_path, id)?;

            // Update index with segment
            let segment_stale_entries = reader.index(&mut index)?;

            stale_entries += segment_stale_entries;

            // update log size
            size += reader.size;

            readers.insert(id.to_string(), reader);
        }

        // Create Log
        Ok(KvStore {
            base_dir: dir_path,
            readers: Arc::new(readers),
            writer: Arc::new(Mutex::new(writer)),
            size: Arc::new(AtomicU64::new(size)),
            stale_entries: Arc::new(AtomicU64::new(stale_entries)),
            index: Arc::new(index),
            compaction: Arc::new(AtomicBool::new(false)),
        })
    }
    fn rollover(&self) -> Result<()> {
        // Create new segment
        let writer_ref = Arc::clone(&self.writer);

        let mut writer = writer_ref.lock().map_err(|_| TsaError::LockPoisoned)?;

        let active_file_id = writer.file_id;

        let new_file_id = 1 + active_file_id;

        let new_writer = SegmentWriter::new(self.base_dir.as_path(), new_file_id)?;
        *writer = new_writer;

        let new_reader = SegmentReader::open(self.base_dir.as_path(), new_file_id)?;
        self.readers.insert(new_file_id.to_string(), new_reader);

        Ok(())
    }
    fn compact(&self) -> Result<()> {
        self.compaction.store(true, Ordering::SeqCst);

        let old_reader_keys: Vec<String> = self
            .readers
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        // Get list of current files
        let keys: Vec<String> = self.index.iter().map(|entry| entry.key().clone()).collect();

        // Reset size so compaction size can be calculated
        self.size.store(0, Ordering::SeqCst);

        // Create new active file to write
        self.rollover()?;

        // Loop through key_dir
        for key in keys {
            // get value
            let value = self
                .get(key.clone())
                .map_err(|_| TsaError::KeyNotFound)?
                .ok_or(TsaError::KeyNotFound)?;

            let mut writer = self.writer.lock().map_err(|_| TsaError::KeyNotFound)?;

            let before_size = writer.size.load(Ordering::Acquire);

            let cmd_pos = writer
                .append(Entry::Set {
                    key: key.clone(),
                    value,
                })
                .map_err(|_| TsaError::KeyNotFound)?;

            let after_size = writer.offset.load(Ordering::Acquire);

            self.size
                .fetch_add(after_size - before_size, Ordering::Relaxed);

            // Update index
            self.index.insert(key, cmd_pos);

            // Check file size
            let writer_size = writer.size().map_err(|_| TsaError::FileNotFound)?;

            if writer_size > MAX_LOG_FILE_SIZE {
                self.rollover()?;
            }
        }

        // Update stale_entries
        self.stale_entries.store(0, Ordering::SeqCst);

        // Drop all file handles before deleting files
        let mut old_readers: Vec<SegmentReader> = vec![];

        for reader_key in &old_reader_keys {
            if let Some((_, value)) = self.readers.remove(reader_key) {
                old_readers.push(value);
            }
        }
        drop(old_readers);

        // Remove old files (active and less)
        for reader_key in old_reader_keys {
            let file_name = format!("{reader_key}.log");
            fs::remove_file(self.base_dir.join(file_name))?;
        }

        self.compaction.store(false, Ordering::SeqCst);

        Ok(())
    }
}

impl StoreTrait for KvStore {
    /// Add a key/value pair to store
    fn set(&self, key: String, value: String) -> Result<()> {
        if value.is_empty() {
            return Err(TsaError::EmptyValue);
        }

        // add value to new file
        let mut writer = self.writer.lock().map_err(|_| TsaError::KeyNotFound)?;

        let before_size = writer.size.load(Ordering::Acquire);

        let cmd_pos = writer
            .append(Entry::Set {
                key: key.clone(),
                value,
            })
            .map_err(|_| TsaError::KeyNotFound)?;

        let after_size = writer.offset.load(Ordering::Acquire);

        self.size
            .fetch_add(after_size - before_size, Ordering::Relaxed);

        // Update index
        if self.index.insert(key, cmd_pos).is_some() {
            // Update stale entries for overwrite
            self.stale_entries.fetch_add(1, Ordering::Relaxed);
        }

        // Check threshold for compaction
        // Prevent recursive compaction
        if self.size.load(Ordering::Acquire) > COMPACTION_THRESHOLD {
            self.compact()?;
            return Ok(());
        }

        // Check file size
        let writer_size = writer.size().map_err(|_| TsaError::FileNotFound)?;

        if writer_size > MAX_LOG_FILE_SIZE {
            self.rollover()?;
        }

        Ok(())
    }
    /// Get a value from store using key
    fn get(&self, key: String) -> Result<Option<String>> {
        // Get log pointer from index

        let log_pointer = match self.index.get(&key) {
            Some(ptr) => ptr,
            None => return Ok(None),
        };

        let file_id = log_pointer.file_id.to_string();

        let readers = Arc::clone(&self.readers);

        let mut reader = match readers.get_mut(&file_id) {
            Some(value) => value,
            None => return Err(TsaError::FileNotFound),
        };

        // Has all the data (kv length, val length, key, value)
        let bytes = reader.read(log_pointer.offset, log_pointer.length)?;

        let entry = Entry::deserialize(&bytes)?;

        if let Entry::Set { value, .. } = entry {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    /// Remove key/value pair from store
    fn remove(&self, key: String) -> Result<()> {
        self.index.remove(&key).ok_or(TsaError::KeyNotFound)?;

        let mut writer = self.writer.lock().map_err(|_| TsaError::KeyNotFound)?;

        let before_size = writer.offset.load(Ordering::Acquire);

        writer
            .append(Entry::Remove { key })
            .map_err(|_| TsaError::KeyNotFound)?;

        let after_size = writer.offset.load(Ordering::Acquire);
        self.size
            .fetch_add(after_size - before_size, Ordering::Relaxed);

        // Update stale entries for removal
        self.stale_entries.fetch_add(1, Ordering::Relaxed);

        // Check threshold for compaction
        if self.size.load(Ordering::Acquire) > COMPACTION_THRESHOLD {
            self.compact()?;
            return Ok(());
        }

        // Check file size
        let writer_size = writer.size().map_err(|_| TsaError::FileNotFound)?;

        if writer_size > MAX_LOG_FILE_SIZE {
            self.rollover()?;
        }

        Ok(())
    }
}
