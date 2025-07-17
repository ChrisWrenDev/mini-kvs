#![deny(missing_docs)]
//! In Memory key/value store.
use super::entry::Entry;
use super::segment::{Segment, SegmentStatus};
use crate::{KvsError, Result, StoreTrait};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

const MAX_LOG_FILE_SIZE: u64 = 4 * 1024 * 1024; // 4 MB
const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1 MB

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
#[derive(Default, Debug, Clone)]
pub struct KvStore {
    base_dir: PathBuf,
    segments: Arc<RwLock<HashMap<String, Arc<RwLock<Segment>>>>>,
    active: Arc<RwLock<String>>,
    size: Arc<AtomicU64>,
    index: Arc<RwLock<HashMap<String, Arc<RwLock<CommandPos>>>>>,
    stale_entries: Arc<AtomicU64>,
    compaction: Arc<AtomicBool>,
}

impl KvStore {
    /// Create a key/value store
    pub fn open(dir_path: PathBuf) -> Result<KvStore> {
        // Add segments to vector
        let mut segments = HashMap::new();

        // Create index
        let mut index = HashMap::new();

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
        let mut active: String = file_ids.iter().copied().max().unwrap_or(0).to_string();

        // Calculate size of log
        let mut size = 0;

        // Calculate stale entries in log
        let mut stale_entries = 0;

        // loop newest to oldest (highest to lowest)
        for id in file_ids {
            // Determine status
            let status = if id.to_string() == active {
                SegmentStatus::Active
            } else {
                SegmentStatus::Archived
            };

            // Create segment for file
            let mut segment = Segment::open(&dir_path, id, status)?;

            // Update index with segment
            let segment_stale_entries = segment.index(&mut index)?;

            // Count stale entries (rm, duplicate)
            if segment_stale_entries > 0 {
                let mut status = segment.status.write().map_err(|_| KvsError::LockPoisoned)?;
                if matches!(*status, SegmentStatus::Archived) {
                    // stale entries = sealed
                    *status = SegmentStatus::Sealed;
                }
            }

            stale_entries += segment_stale_entries;

            // update log size
            size += segment.size.load(Ordering::Acquire);

            // Add segment to segments hashmap
            segments.insert(id.to_string(), Arc::new(RwLock::new(segment)));
        }

        // If no files, create one
        if segments.is_empty() {
            let file_id = 1;
            active = format!("{file_id}");
            let segment = Segment::new(&dir_path, file_id)?;
            segments.insert(active.clone(), Arc::new(RwLock::new(segment)));
        }

        // Create Log
        Ok(KvStore {
            base_dir: dir_path,
            segments: Arc::new(RwLock::new(segments)),
            active: Arc::new(RwLock::new(active)),
            size: Arc::new(AtomicU64::new(size)),
            stale_entries: Arc::new(AtomicU64::new(stale_entries)),
            index: Arc::new(RwLock::new(index)),
            compaction: Arc::new(AtomicBool::new(false)),
        })
    }
    fn rollover(&self) -> Result<()> {
        // Create new segment
        let active_file_id = self
            .active
            .read()
            .map_err(|_| KvsError::LockPoisoned)? // or just unwrap if using `parking_lot`
            .parse::<u64>()
            .map_err(|_| KvsError::FileNotFound)?;

        let new_file_id = 1 + active_file_id;

        let new_segment = Arc::new(RwLock::new(
            Segment::new(self.base_dir.as_path(), new_file_id)
                .map_err(|_| KvsError::FileNotFound)?,
        ));

        self.segments
            .write()
            .map_err(|_| KvsError::LockPoisoned)?
            .insert(new_file_id.to_string(), new_segment);

        // Update active segment
        *self.active.write().map_err(|_| KvsError::LockPoisoned)? = new_file_id.to_string();

        Ok(())
    }
    fn compact(&self) -> Result<()> {
        self.compaction.store(true, Ordering::SeqCst);

        let old_segment_keys: Vec<String> = self
            .segments
            .read()
            .map_err(|_| KvsError::LockPoisoned)?
            .keys()
            .cloned()
            .collect();

        // Get list of current files
        let keys: Vec<String> = self
            .index
            .read()
            .map_err(|_| KvsError::LockPoisoned)?
            .keys()
            .cloned()
            .collect();

        // Reset size so compaction size can be calculated
        self.size.store(0, Ordering::SeqCst);

        // Create new active file to write
        self.rollover()?;

        // Loop through key_dir
        for key in keys {
            // get value
            let value = self
                .get(key.clone())
                .map_err(|_| KvsError::KeyNotFound)?
                .ok_or(KvsError::KeyNotFound)?;

            let active_key = self
                .active
                .read()
                .map_err(|_| KvsError::LockPoisoned)?
                .clone();

            let mut segments = self.segments.write().map_err(|_| KvsError::LockPoisoned)?;

            // add value to new file
            let mut active_segment = segments
                .get_mut(&active_key)
                .ok_or(KvsError::FileNotFound)?
                .write()
                .map_err(|_| KvsError::LockPoisoned)?;

            let before_size = active_segment.size.load(Ordering::Acquire);

            let cmd_pos = active_segment
                .append(Entry::Set {
                    key: key.clone(),
                    value,
                })
                .map_err(|_| KvsError::KeyNotFound)?;

            let after_size = active_segment.offset.load(Ordering::Acquire);
            self.size
                .fetch_add(after_size - before_size, Ordering::Relaxed);

            // Update index
            self.index
                .write()
                .map_err(|_| KvsError::LockPoisoned)?
                .insert(key, Arc::new(RwLock::new(cmd_pos)));

            // Check file size
            let segment_size = active_segment.size().map_err(|_| KvsError::FileNotFound)?;

            if segment_size > MAX_LOG_FILE_SIZE {
                self.rollover()?;
            }
        }

        // Update stale_entries
        self.stale_entries.store(0, Ordering::SeqCst);

        // Drop all file handles before deleting files
        let mut old_segments: Vec<Arc<RwLock<Segment>>> = vec![];

        let mut segments = self.segments.write().map_err(|_| KvsError::LockPoisoned)?;

        for segment_key in &old_segment_keys {
            if let Some(seg) = segments.remove(segment_key) {
                old_segments.push(seg);
            }
        }
        drop(old_segments);

        // Remove old files (active and less)
        for segment_key in old_segment_keys {
            let file_name = format!("{segment_key}.log");
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
            return Err(KvsError::EmptyValue);
        }

        let active_key = self
            .active
            .read()
            .map_err(|_| KvsError::LockPoisoned)?
            .clone();

        let mut segments = self.segments.write().map_err(|_| KvsError::LockPoisoned)?;

        // add value to new file
        let mut active_segment = segments
            .get_mut(&active_key)
            .ok_or(KvsError::FileNotFound)?
            .write()
            .map_err(|_| KvsError::LockPoisoned)?;

        let before_size = active_segment.size.load(Ordering::Acquire);

        let cmd_pos = active_segment
            .append(Entry::Set {
                key: key.clone(),
                value,
            })
            .map_err(|_| KvsError::KeyNotFound)?;

        let after_size = active_segment.offset.load(Ordering::Acquire);
        self.size
            .fetch_add(after_size - before_size, Ordering::Relaxed);

        // Update index
        if self
            .index
            .write()
            .map_err(|_| KvsError::LockPoisoned)?
            .insert(key, Arc::new(RwLock::new(cmd_pos)))
            .is_some()
        {
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
        let segment_size = active_segment.size().map_err(|_| KvsError::FileNotFound)?;

        if segment_size > MAX_LOG_FILE_SIZE {
            self.rollover()?;
        }

        Ok(())
    }
    /// Get a value from store using key
    fn get(&self, key: String) -> Result<Option<String>> {
        // Get log pointer from index
        let index = self.index.read().map_err(|_| KvsError::LockPoisoned)?;

        let log_pointer = match index.get(&key) {
            Some(ptr) => ptr,
            None => return Ok(None),
        };

        let log_pointer = log_pointer.read().map_err(|_| KvsError::LockPoisoned)?;

        let file_id = log_pointer.file_id.to_string();

        let mut segments = self.segments.write().map_err(|_| KvsError::LockPoisoned)?;

        let mut segment = segments
            .get_mut(&file_id)
            .ok_or(KvsError::FileNotFound)?
            .write()
            .map_err(|_| KvsError::LockPoisoned)?;

        // Has all the data (kv length, val length, key, value)
        let bytes = segment.read(log_pointer.offset, log_pointer.length)?;

        let entry = Entry::deserialize(&bytes)?;

        if let Entry::Set { value, .. } = entry {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    /// Remove key/value pair from store
    fn remove(&self, key: String) -> Result<()> {
        self.index
            .write()
            .map_err(|_| KvsError::LockPoisoned)?
            .remove(&key)
            .ok_or(KvsError::KeyNotFound)?;

        let active_key = self
            .active
            .read()
            .map_err(|_| KvsError::LockPoisoned)?
            .clone();

        let mut segments = self.segments.write().map_err(|_| KvsError::LockPoisoned)?;

        let mut active_segment = segments
            .get_mut(&active_key)
            .ok_or(KvsError::FileNotFound)?
            .write()
            .map_err(|_| KvsError::LockPoisoned)?;

        let before_size = active_segment.offset.load(Ordering::Acquire);

        active_segment
            .append(Entry::Remove { key })
            .map_err(|_| KvsError::KeyNotFound)?;

        let after_size = active_segment.offset.load(Ordering::Acquire);
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
        let segment_size = active_segment.size().map_err(|_| KvsError::FileNotFound)?;

        if segment_size > MAX_LOG_FILE_SIZE {
            self.rollover()?;
        }

        Ok(())
    }
}

// impl Clone for KvStore {
//     fn clone(&self) -> Self {
//         KvStore { ..self.clone() }
//     }
// }
