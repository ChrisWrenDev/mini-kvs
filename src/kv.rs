#![deny(missing_docs)]
//! In Memory key/value store.

use std::collections::HashMap;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::Result;

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
    fn new(dir_path: &PathBuf, file_id: u64) -> Result<Segment> {
        // Create empty file
        let file_name = format!("log.{}", file_id);
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
    fn open(dir_path: &PathBuf, file_id: u64, status: SegmentStatus) -> Result<Segment> {
        // Create empty file
        let file_name = format!("log.{}", file_id);
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
    fn read(&mut self, offset: u64, length: u64) -> Result<String> {
        // Get key-value at a given offset (provided by index)

        let mut buffer = vec![0; length as usize];

        // Find file offset
        self.reader.seek(SeekFrom::Start(offset))?;

        // Read bytes
        self.reader.read_exact(&mut buffer)?;

        // Deserialise value
        // Return value
        Ok(String::from_utf8_lossy(&buffer).to_string())
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

        // Update segment offset
        self.offset += buffer.len() as u64;

        // Update file size
        self.size = self.size()?;

        // Update log pointer
        Ok(CommandPos {
            file_id: self.file_id,
            offset: cur_offset,
            length: buffer.len() as u64,
        })
    }
    fn size(&self) -> Result<u64> {
        // Measure if compaction is needed
        let file_ref = self.reader.get_ref();
        let metadata = file_ref.metadata()?;
        let size = metadata.len();
        Ok(size)
    }
    fn compact() {
        // Remove stale entries
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
    pub fn open(dir_path: String) -> KvStore {
        let base_dir = PathBuf::from(dir_path);

        // Add segments to vector
        let mut segments = HashMap::new();

        // Create index
        let index = HashMap::new();

        // Temp - Single File
        let file_id = 1;
        let active = file_id.to_owned().to_string();
        let segment = Segment::new(&base_dir, file_id).unwrap();
        segments.insert(active.clone(), segment);

        // Check directory for log files
        // Create segment for each log file
        // Update index with segment

        // Create Log
        KvStore {
            base_dir,
            segments,
            active,
            size: 0,
            index,
        }
    }
    /// Add a key/value pair to store
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // Create set value
        // Serialize value to string
        // Append to log
        // If successful exit silently
        // If failed print error / return non-zero code

        let segment_size = self.segments.get_mut(&self.active).unwrap().size().unwrap();

        if segment_size >= MAX_LOG_FILE_SIZE {
            let active_segment = self.segments.get_mut(&self.active).unwrap();

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

        let active_segment = self.segments.get_mut(&self.active).unwrap();

        let cmd_pos = active_segment.append(key.clone(), value).unwrap();

        // Update index
        self.index.insert(key, cmd_pos);

        Ok(())
    }
    /// Get a value from store using key
    pub fn get(&mut self, key: String) -> Result<String> {
        // Read log to build index (key + log pointer)
        // check index for key
        // If succcessful deserialise and print value
        // If failed print "Key not found"
        // exit code 0

        // Get log pointer from index
        let log_pointer = self.index.get(&key).unwrap();

        let file_id = log_pointer.file_id.to_string();

        // Get key-value from segment
        let segment = self.segments.get_mut(&file_id).unwrap();

        let value = segment
            .read(log_pointer.offset, log_pointer.length)
            .unwrap();

        Ok(value)
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
