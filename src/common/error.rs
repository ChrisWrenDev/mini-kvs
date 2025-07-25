use failure::Fail;

#[derive(Fail, Debug)]
pub enum TsaError {
    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] std::io::Error),

    #[fail(display = "Serialization or deserialization error: {}", _0)]
    Serde(#[cause] serde_json::Error),

    #[fail(display = "Deserialization error: {}", _0)]
    Toml(#[cause] toml::de::Error),

    #[fail(display = "Invalid UTF-8: {}", _0)]
    Utf8(#[cause] std::str::Utf8Error),

    #[fail(display = "UTF-8 error: {}", _0)]
    FromUtf8(#[cause] std::string::FromUtf8Error),

    #[fail(display = "From Int error: {}", _0)]
    FromInt(#[cause] std::num::TryFromIntError),

    #[fail(display = "sled error: {}", _0)]
    Sled(#[cause] sled::Error),

    #[fail(display = "rayon error: {}", _0)]
    Rayon(#[cause] rayon::ThreadPoolBuildError),

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

    #[fail(display = "Protocol error: {}", _0)]
    Protocol(String),

    #[fail(display = "Lock error: {}", _0)]
    LockError(String),

    #[fail(display = "Lock poison error")]
    LockPoisoned,
}

pub type Result<T> = std::result::Result<T, TsaError>;

impl From<std::io::Error> for TsaError {
    fn from(err: std::io::Error) -> Self {
        TsaError::Io(err)
    }
}

impl From<serde_json::Error> for TsaError {
    fn from(err: serde_json::Error) -> Self {
        TsaError::Serde(err)
    }
}

impl From<toml::de::Error> for TsaError {
    fn from(err: toml::de::Error) -> Self {
        TsaError::Toml(err)
    }
}

impl From<std::str::Utf8Error> for TsaError {
    fn from(err: std::str::Utf8Error) -> Self {
        TsaError::Utf8(err)
    }
}

impl From<std::string::FromUtf8Error> for TsaError {
    fn from(err: std::string::FromUtf8Error) -> TsaError {
        TsaError::FromUtf8(err)
    }
}

impl From<std::num::TryFromIntError> for TsaError {
    fn from(err: std::num::TryFromIntError) -> TsaError {
        TsaError::FromInt(err)
    }
}

impl From<sled::Error> for TsaError {
    fn from(err: sled::Error) -> TsaError {
        TsaError::Sled(err)
    }
}

impl From<rayon::ThreadPoolBuildError> for TsaError {
    fn from(err: rayon::ThreadPoolBuildError) -> TsaError {
        TsaError::Rayon(err)
    }
}

impl From<String> for TsaError {
    fn from(s: String) -> Self {
        TsaError::Protocol(s)
    }
}

impl From<&str> for TsaError {
    fn from(s: &str) -> Self {
        TsaError::Protocol(s.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, sled::Db>>> for TsaError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, sled::Db>>) -> Self {
        TsaError::LockError(err.to_string())
    }
}

impl
    From<
        std::sync::PoisonError<
            std::sync::MutexGuard<'_, std::collections::HashMap<String, String>>,
        >,
    > for TsaError
{
    fn from(
        err: std::sync::PoisonError<
            std::sync::MutexGuard<'_, std::collections::HashMap<String, String>>,
        >,
    ) -> Self {
        TsaError::LockError(err.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, std::io::BufReader<std::fs::File>>>>
    for TsaError
{
    fn from(
        err: std::sync::PoisonError<std::sync::MutexGuard<'_, std::io::BufReader<std::fs::File>>>,
    ) -> Self {
        TsaError::LockError(err.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, std::io::BufWriter<std::fs::File>>>>
    for TsaError
{
    fn from(
        err: std::sync::PoisonError<std::sync::MutexGuard<'_, std::io::BufWriter<std::fs::File>>>,
    ) -> Self {
        TsaError::LockError(err.to_string())
    }
}
