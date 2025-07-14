#![deny(missing_docs)]
//! In Memory key/value store.

use std::collections::HashMap;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::{KvsError, Result, StoreTrait};

const MAX_LOG_FILE_SIZE: u64 = 4 * 1024 * 1024; // 4 MB
const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1 MB

#[derive(Debug)]
enum Entry {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Entry {
    fn serialize(&self) -> Vec<u8> {
        let (key_bytes, value_bytes) = match self {
            Entry::Set { key, value } => (key.as_bytes(), value.as_bytes()),
            Entry::Remove { key } => (key.as_bytes(), &[][..]),
        };

        // key_size
        let key_size = key_bytes.len().to_le_bytes();
        // value_size
        let value_size = value_bytes.len().to_le_bytes();

        // [ksz, vsz, key, value]
        let mut buffer: Vec<u8> = Vec::with_capacity(16 + key_bytes.len() + value_bytes.len());

        buffer.extend_from_slice(&key_size);
        buffer.extend_from_slice(&value_size);
        buffer.extend_from_slice(key_bytes);
        if !value_bytes.is_empty() {
            buffer.extend_from_slice(value_bytes);
        }

        buffer
    }
    fn deserialize(mut bytes: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        let mut ksz_buf = [0u8; 8];
        let mut vsz_buf = [0u8; 8];

        bytes.read_exact(&mut ksz_buf)?;
        bytes.read_exact(&mut vsz_buf)?;

        let key_size = u64::from_le_bytes(ksz_buf);
        let value_size = u64::from_le_bytes(vsz_buf);

        let mut key_bytes = vec![0; key_size as usize];
        let mut value_bytes = vec![0; value_size as usize];

        bytes.read_exact(&mut key_bytes)?;
        bytes.read_exact(&mut value_bytes)?;

        let key = String::from_utf8(key_bytes)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in key"))?;
        let value = String::from_utf8(value_bytes)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in value"))?;

        if value.is_empty() {
            Ok(Entry::Remove { key })
        } else {
            Ok(Entry::Set { key, value })
        }
    }
}

#[derive(Debug)]
enum SegmentStatus {
    Active, // Currently being written to
    Sealed, // Closed to writes, may be compacted
    // Compacting, // Being compacted into a new segment
    Archived, // Fully compacted, no longer in use
}

#[derive(Debug)]
struct Segment {
    file_id: u64,
    offset: u64,
    size: u64,
    status: SegmentStatus,
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl Segment {
    fn new(dir_path: &Path, file_id: u64) -> Result<Segment> {
        // Create empty file
        let file_name = format!("{file_id}.log");
        let path = dir_path.join(file_name);

        // Write handle
        let writer_file = OpenOptions::new()
            .write(true)
            .create_new(true) // Fail if file exists
            .open(&path)?;

        // Reader handle
        let reader_file = OpenOptions::new().read(true).open(&path)?;

        // File size
        let metadata = writer_file.metadata()?;
        let size = metadata.len();

        // Create Segment
        Ok(Segment {
            file_id,
            offset: size,
            size,
            status: SegmentStatus::Active,
            reader: BufReader::new(reader_file),
            writer: BufWriter::new(writer_file),
        })
    }
    fn open(dir_path: &Path, file_id: u64, status: SegmentStatus) -> Result<Segment> {
        // Create empty file
        let file_name = format!("{file_id}.log");
        let path = dir_path.join(file_name);

        // Write handle
        let mut writer_file = OpenOptions::new().write(true).open(&path)?;

        // Reader handle
        let reader_file = OpenOptions::new().read(true).open(&path)?;

        // Seek the writer to the end of file
        let size = writer_file.seek(SeekFrom::End(0))?;

        // Create Segment
        Ok(Segment {
            file_id,
            offset: size,
            size,
            status,
            reader: BufReader::new(reader_file),
            writer: BufWriter::new(writer_file),
        })
    }
    fn read(&mut self, offset: u64, length: u64) -> Result<Vec<u8>> {
        // Get key-value at a given offset (provided by index)

        let mut buffer = vec![0; length as usize];

        // Find file offset
        self.reader.seek(SeekFrom::Start(offset))?;

        // Read bytes
        self.reader.read_exact(&mut buffer)?;

        // Deserialise value
        // Return value
        Ok(buffer)
    }
    fn append(&mut self, entry: Entry) -> Result<CommandPos> {
        // Add key-value and return offset for index

        if let Entry::Set { ref value, .. } = entry {
            if value.is_empty() {
                return Err(KvsError::EmptyValue);
            }
        }
        // Current Segment Offset
        let cur_offset = self.offset;

        let buffer = entry.serialize();

        // Write to file
        self.writer.write_all(&buffer)?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        // Update segment offset
        self.offset += buffer.len() as u64;

        // Ensure segment size reflects file size growth
        self.size = self.offset;

        // Update log pointer
        Ok(CommandPos {
            file_id: self.file_id,
            offset: cur_offset,
            length: buffer.len() as u64,
        })
    }
    fn index(&mut self, index: &mut HashMap<String, CommandPos>) -> Result<u64> {
        let mut stale_entries = 0;
        let mut read_offset = 0;
        let current_size = self.size()?;

        // loop through file
        while read_offset + 8 + 8 <= current_size {
            // Calculate size of entry
            let size_buffer = match self.read(read_offset, 16) {
                Ok(e) => e,
                Err(_) => break,
            };

            let key_size = u64::from_le_bytes(size_buffer[0..8].try_into().unwrap());
            let value_size = u64::from_le_bytes(size_buffer[8..16].try_into().unwrap());

            let entry_len = 16 + key_size + value_size;

            // Read buffer (slightly larger than Entry)
            let buffer = match self.read(read_offset, entry_len) {
                Ok(buf) => buf,
                Err(_) => break, // incomplete or corrupted entry
            };

            // Deserialize Entry
            let slice = buffer.as_slice();
            let entry = match Entry::deserialize(slice) {
                Ok(e) => e,
                Err(_) => break,
            };

            // Size of Entry
            let mut consumed = 8u64 + 8u64; // ksz + vsz

            match &entry {
                Entry::Set { key, value } => {
                    let entry_length = key.len() as u64 + value.len() as u64;
                    consumed += entry_length;

                    if index
                        .insert(
                            key.clone(),
                            CommandPos {
                                file_id: self.file_id,
                                offset: read_offset,
                                length: consumed,
                            },
                        )
                        .is_some()
                    {
                        stale_entries += 1;
                    }
                }
                Entry::Remove { key } => {
                    consumed += key.len() as u64;
                    if index.remove(key).is_some() {
                        stale_entries += 1;
                    }
                }
            }

            // Update read offset
            read_offset += consumed;
        }

        Ok(stale_entries)
    }
    fn size(&self) -> Result<u64> {
        // Measure if compaction is needed
        let file_ref = self.reader.get_ref();
        let metadata = file_ref.metadata()?;
        let size = metadata.len();
        Ok(size)
    }
}

#[derive(Debug)]
struct CommandPos {
    file_id: u64, // Which file
    offset: u64,  // Where in the file
    length: u64,  // Number of bytes
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
#[derive(Default)]
pub struct KvStore {
    base_dir: PathBuf,
    segments: HashMap<String, Segment>,
    active: String,
    size: u64,
    index: HashMap<String, CommandPos>,
    stale_entries: u64,
    compaction: bool,
}

impl KvStore {
    /// Create a key/value store
    pub fn open(dir_path: &Path) -> Result<KvStore> {
        let base_dir = PathBuf::from(dir_path);

        // Add segments to vector
        let mut segments = HashMap::new();

        // Create index
        let mut index = HashMap::new();

        // Get all files
        // Check directory for log files
        let mut file_ids = fs::read_dir(dir_path)?
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
            let mut segment = Segment::open(dir_path, id, status)?;

            // Update index with segment
            let segment_stale_entries = segment.index(&mut index)?;

            // Count stale entries (rm, duplicate)
            if matches!(segment.status, SegmentStatus::Archived) && segment_stale_entries > 0 {
                // stale entries = sealed
                segment.status = SegmentStatus::Sealed;
            }

            stale_entries += segment_stale_entries;

            // update log size
            size += segment.size;

            // Add segment to segments hashmap
            segments.insert(id.to_string(), segment);
        }

        // If no files, create one
        if segments.is_empty() {
            let file_id = 1;
            active = format!("{file_id}");
            let segment = Segment::new(&base_dir, file_id)?;
            segments.insert(active.clone(), segment);
        }

        // Create Log
        Ok(KvStore {
            base_dir,
            segments,
            active,
            size,
            stale_entries,
            index,
            compaction: false,
        })
    }
    fn rollover(&mut self) -> Result<()> {
        // Create new segment
        let active_file_id = self
            .active
            .parse::<u64>()
            .map_err(|_| KvsError::FileNotFound)?;

        let new_file_id = 1 + active_file_id;

        let new_segment = Segment::new(self.base_dir.as_path(), new_file_id)
            .map_err(|_| KvsError::FileNotFound)?;

        self.segments.insert(new_file_id.to_string(), new_segment);

        // Update active segment
        self.active = new_file_id.to_string();

        Ok(())
    }
    fn compact(&mut self) -> Result<()> {
        self.compaction = true;

        let old_segment_keys: Vec<String> = self.segments.keys().cloned().collect();

        // Get list of current files
        let keys: Vec<String> = self.index.keys().cloned().collect();

        // Reset size so compaction size can be calculated
        self.size = 0;

        // Create new active file to write
        self.rollover()?;

        // Loop through key_dir
        for key in keys {
            // get value
            let value = self
                .get(key.clone())
                .map_err(|_| KvsError::KeyNotFound)?
                .ok_or(KvsError::KeyNotFound)?;

            // add value to new file
            let active_segment = self
                .segments
                .get_mut(&self.active)
                .ok_or(KvsError::FileNotFound)?;

            let before_size = active_segment.size;

            let cmd_pos = active_segment
                .append(Entry::Set {
                    key: key.clone(),
                    value,
                })
                .map_err(|_| KvsError::KeyNotFound)?;

            let after_size = active_segment.offset;
            self.size += after_size - before_size;

            // Update index
            self.index.insert(key, cmd_pos);

            // Check file size
            let segment_size = active_segment.size().map_err(|_| KvsError::FileNotFound)?;

            if segment_size > MAX_LOG_FILE_SIZE {
                self.rollover()?;
            }
        }

        // Update stale_entries
        self.stale_entries = 0;

        // Drop all file handles before deleting files
        let mut old_segments: Vec<Segment> = vec![];

        for segment_key in &old_segment_keys {
            if let Some(seg) = self.segments.remove(segment_key) {
                old_segments.push(seg);
            }
        }
        drop(old_segments);

        // Remove old files (active and less)
        for segment_key in old_segment_keys {
            let file_name = format!("{segment_key}.log");
            fs::remove_file(self.base_dir.join(file_name))?;
        }

        self.compaction = false;

        Ok(())
    }
}

impl StoreTrait for KvStore {
    /// Add a key/value pair to store
    fn set(&mut self, key: String, value: String) -> Result<()> {
        if value.is_empty() {
            return Err(KvsError::EmptyValue);
        }

        let active_segment = self
            .segments
            .get_mut(&self.active)
            .ok_or(KvsError::FileNotFound)?;

        let before_size = active_segment.size;

        let cmd_pos = active_segment
            .append(Entry::Set {
                key: key.clone(),
                value,
            })
            .map_err(|_| KvsError::KeyNotFound)?;

        let after_size = active_segment.offset;
        self.size += after_size - before_size;

        // Update index
        if self.index.insert(key, cmd_pos).is_some() {
            // Update stale entries for overwrite
            self.stale_entries += 1;
        }

        // Check threshold for compaction
        // Prevent recursive compaction
        if self.size > COMPACTION_THRESHOLD {
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
    fn get(&mut self, key: String) -> Result<Option<String>> {
        // Get log pointer from index
        let log_pointer = match self.index.get(&key) {
            Some(ptr) => ptr,
            None => return Ok(None),
        };

        let file_id = log_pointer.file_id.to_string();

        let segment = match self.segments.get_mut(&file_id) {
            Some(seg) => seg,
            None => return Ok(None),
        };

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
    fn remove(&mut self, key: String) -> Result<()> {
        self.index.remove(&key).ok_or(KvsError::KeyNotFound)?;

        let active_segment = self
            .segments
            .get_mut(&self.active)
            .ok_or(KvsError::FileNotFound)?;

        let before_size = active_segment.offset;

        active_segment
            .append(Entry::Remove { key })
            .map_err(|_| KvsError::KeyNotFound)?;

        let after_size = active_segment.offset;
        self.size += after_size - before_size;

        // Update stale entries for removal
        self.stale_entries += 1;

        // Check threshold for compaction
        if self.size > COMPACTION_THRESHOLD {
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
