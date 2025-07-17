use super::entry::Entry;
use super::store::CommandPos;
use crate::{KvsError, Result};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum SegmentStatus {
    Active, // Currently being written to
    Sealed, // Closed to writes, may be compacted
    // Compacting, // Being compacted into a new segment
    Archived, // Fully compacted, no longer in use
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub file_id: u64,
    pub offset: u64,
    pub size: u64,
    pub status: SegmentStatus,
    pub reader: Arc<Mutex<BufReader<File>>>,
    pub writer: Arc<Mutex<BufWriter<File>>>,
}

impl Segment {
    pub fn new(dir_path: &Path, file_id: u64) -> Result<Segment> {
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
            reader: Arc::new(Mutex::new(BufReader::new(reader_file))),
            writer: Arc::new(Mutex::new(BufWriter::new(writer_file))),
        })
    }
    pub fn open(dir_path: &Path, file_id: u64, status: SegmentStatus) -> Result<Segment> {
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
            reader: Arc::new(Mutex::new(BufReader::new(reader_file))),
            writer: Arc::new(Mutex::new(BufWriter::new(writer_file))),
        })
    }
    pub fn read(&mut self, offset: u64, length: u64) -> Result<Vec<u8>> {
        // Get key-value at a given offset (provided by index)

        let mut buffer = vec![0; length as usize];

        // Find file offset
        self.reader.lock()?.seek(SeekFrom::Start(offset))?;

        // Read bytes
        self.reader.lock()?.read_exact(&mut buffer)?;

        // Deserialise value
        // Return value
        Ok(buffer)
    }
    pub fn append(&mut self, entry: Entry) -> Result<CommandPos> {
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
        self.writer.lock()?.write_all(&buffer)?;
        self.writer.lock()?.flush()?;
        self.writer.lock()?.get_ref().sync_data()?;

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
    pub fn index(&mut self, index: &mut HashMap<String, CommandPos>) -> Result<u64> {
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
    pub fn size(&self) -> Result<u64> {
        // Measure if compaction is needed
        let binding = self.reader.lock()?;
        let file_ref = binding.get_ref();
        let metadata = file_ref.metadata()?;
        let size = metadata.len();
        Ok(size)
    }
}
