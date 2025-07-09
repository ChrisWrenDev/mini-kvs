// looks for file or folder with mod.rs
mod kvmemory;
mod kvstore;

// expose through storage module
pub use kvmemory::KvMemory;
pub use kvstore::KvStore;
