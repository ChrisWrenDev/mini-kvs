#![deny(missing_docs)]
//! In Memory key/value store.

use std::collections::HashMap;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::{KvsError, Result};

const MAX_LOG_FILE_SIZE: u64 = 4 * 1024 * 1024; // 4 MB

enum SegmentStatus {
    Active,     // Currently being written to
    Sealed,     // Closed to writes, may be compacted
    Compacting, // Being compacted into a new segment
    Archived,   // Fully compacted, no longer in use
}

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
        let file_name = format!("{}.log", file_id);
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
        let file_name = format!("{}.log", file_id);
        let path = dir_path.join(file_name);

        // Write handle
        let writer_file = OpenOptions::new().write(true).open(&path)?;

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
            status,
            reader: BufReader::new(reader_file),
            writer: BufWriter::new(writer_file),
        })
    }
    fn read(&mut self, offset: u64, length: u64) -> Result<Vec<u8>> {
        // Get key-value at a given offset (provided by index)

        let file_len = self.reader.get_ref().metadata()?.len();
        println!(
            "Offset: {}, Length: {}, File size: {}",
            offset, length, file_len
        );

        let mut buffer = vec![0; length as usize];

        // Find file offset
        self.reader.seek(SeekFrom::Start(offset))?;

        // Read bytes
        self.reader.read_exact(&mut buffer)?;

        // Deserialise value
        // Return value
        Ok(buffer)
    }
    fn append(&mut self, key: String, value: String) -> Result<CommandPos> {
        // Add key-value and return offset for index

        // Current Segment Offset
        let cur_offset = self.offset;

        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();

        // key_size
        let key_size = key_bytes.len().to_le_bytes();
        // value_size
        let value_size = value_bytes.len().to_le_bytes();

        // [ksz, vsz, key, value]
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend_from_slice(&key_size);
        buffer.extend_from_slice(&value_size);
        buffer.extend_from_slice(key_bytes);
        buffer.extend_from_slice(value_bytes);

        // Serialise value

        // Write to file
        self.writer.write_all(&buffer)?;
        self.writer.flush()?;

        // Value offset
        let value_offset = cur_offset + 16 + key_bytes.len() as u64;

        // Update segment offset
        self.offset += buffer.len() as u64;

        // Update file size
        self.size = self.size()?;

        // Update log pointer
        Ok(CommandPos {
            file_id: self.file_id,
            offset: value_offset,
            length: value_bytes.len() as u64,
        })
    }
    fn index(&mut self, index: &mut HashMap<String, CommandPos>) -> Result<u32> {
        let mut offset = 0;
        let mut stale_entries = 0;
        let file_size = self.reader.get_ref().metadata()?.len();

        // loop through file
        loop {
            if offset + 16 > file_size {
                break; // not enough for header
            }

            // Get key/value size
            let key_size_bytes: [u8; 8] = self
                .read(offset, 8)?
                .try_into()
                .expect("Expected exactly 8 bytes");
            let key_size = u64::from_le_bytes(key_size_bytes);

            offset += 8;

            let value_size_bytes: [u8; 8] = self
                .read(offset, 8)?
                .try_into()
                .expect("Expected exactly 8 bytes");
            let value_size = u64::from_le_bytes(value_size_bytes);

            offset += 8;

            if offset + key_size + value_size > file_size {
                break; // incomplete entry
            }

            // Get key value
            let key_bytes = match self.read(offset, key_size) {
                Ok(bytes) => bytes,
                _ => break, // EOF or read error
            };

            let key_value = match String::from_utf8(key_bytes) {
                Ok(s) => s,
                Err(_) => break, // Skip invalid UTF-8 key
            };

            // Update offset to start of value
            offset += key_size;

            // Handle removed key/value pairs (tombstone value)
            if value_size == 0 {
                stale_entries += 1;
                continue;
            }

            // Update index
            index.insert(
                key_value,
                CommandPos {
                    file_id: self.file_id,
                    offset,
                    length: value_size,
                },
            );

            // Update offset to start of next entry
            offset += value_size;
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
    fn compact(&mut self) {
        // Remove stale entries
        self.status = SegmentStatus::Compacting;
    }
}

struct CommandPos {
    file_id: u64, // Which file
    offset: u64,  // Where in the file
    length: u64,  // Number of bytes
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
    base_dir: PathBuf,
    segments: HashMap<String, Segment>,
    active: String,
    size: u64,
    index: HashMap<String, CommandPos>,
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
            let stale_entries = segment.index(&mut index)?;

            // Count stale entries (rm, duplicate)
            match segment.status {
                SegmentStatus::Archived if stale_entries > 0 => {
                    // stale entires = sealed
                    segment.status = SegmentStatus::Sealed;
                }
                _ => {}
            }

            // update log size
            size += segment.size;

            // Add segment to segments hashmap
            segments.insert(id.to_string(), segment);
        }

        // If no files, create one
        if segments.is_empty() {
            let file_id = 1;
            active = file_id.to_owned().to_string();
            let segment = Segment::new(&base_dir, file_id)?;
            segments.insert(active.clone(), segment);
        }

        // Create Log
        Ok(KvStore {
            base_dir,
            segments,
            active,
            size,
            index,
        })
    }
    /// Add a key/value pair to store
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // Create set value
        // Serialize value to string
        // Append to log
        // If successful exit silently
        // If failed print error / return non-zero code

        let segment_size = self
            .segments
            .get_mut(&self.active)
            .ok_or(KvsError::FileNotFound)?
            .size()
            .map_err(|_| KvsError::FileNotFound)?;

        if segment_size >= MAX_LOG_FILE_SIZE {
            let active_segment = self
                .segments
                .get_mut(&self.active)
                .ok_or(KvsError::FileNotFound)?;

            // Change status
            active_segment.status = SegmentStatus::Sealed;

            // Create new segment
            let active_file_id = self.active.parse::<u64>().unwrap();
            let new_file_id = 1 + active_file_id;
            let segment = Segment::new(&self.base_dir, new_file_id).unwrap();
            self.segments.insert(new_file_id.to_string(), segment);

            // Update active segment
            self.active = new_file_id.to_string();
        }

        let active_segment = self
            .segments
            .get_mut(&self.active)
            .ok_or(KvsError::FileNotFound)?;

        let cmd_pos = active_segment
            .append(key.clone(), value)
            .map_err(|_| KvsError::KeyNotFound)?;

        // Update index
        self.index.insert(key, cmd_pos);

        Ok(())
    }
    /// Get a value from store using key
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // Read log to build index (key + log pointer)
        // check index for key
        // If succcessful deserialise and print value
        // If failed print "Key not found"
        // exit code 0

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
        let value_bytes = segment.read(log_pointer.offset, log_pointer.length)?;
        let value = String::from_utf8_lossy(&value_bytes).into_owned();

        Ok(Some(value))
    }
    /// Remove key/value pair from store
    pub fn remove(&mut self, key: String) -> Result<()> {
        // Read log to build index (key + log pointer)
        // check index for key
        // If fail print "Key not found"
        // If successful
        // -- create rm value
        // -- serialize value to string
        // -- append to log
        // -- if successful exit silently / exit code 0
        // -- if failure print error / return non-zero code
        self.index.remove(&key).ok_or(KvsError::KeyNotFound)?;
        self.set(key.clone(), "".to_string())?;
        Ok(())
    }
    /// Size of log
    pub fn size(self) -> u64 {
        self.size
    }
}
