use failure::Fail;
use serde_json;
use std::io;

#[derive(Fail, Debug)]
pub enum KvsError {
    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "Serialization or deserialization error: {}", _0)]
    Serde(#[cause] serde_json::Error),

    #[fail(display = "Key not found")]
    KeyNotFound,

    #[fail(display = "File not found")]
    FileNotFound,

    #[fail(display = "Empty values not allowed")]
    EmptyValue,

    #[fail(display = "Log compaction error: {}", _0)]
    Compaction(String),

    #[fail(display = "Unexpected log entry or command: {}", _0)]
    UnexpectedCommand(String),

    #[fail(display = "Engine mismatch error")]
    WrongEngine,

    #[fail(display = "Concurrency error: {}", _0)]
    Concurrency(String),

    #[fail(display = "Corrupted or incomplete log data")]
    CorruptedLog,
}

pub type Result<T> = std::result::Result<T, KvsError>;

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> Self {
        KvsError::Serde(err)
    }
}
