use std::io::Read;

#[derive(Debug)]
pub enum Entry {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Entry {
    pub fn serialize(&self) -> Vec<u8> {
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
    pub fn deserialize(mut bytes: &[u8]) -> std::io::Result<Self> {
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
