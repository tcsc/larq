use crate::key::Key;
use bitflags::bitflags;
use trait_async::trait_async;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NoSuchObject,
    AccessDenied,
    NetworkError,
    UnknownError,
}

/**
 * An abstract storage object managed by a storage service.
 */
#[derive(Debug)]
pub struct ObjectInfo {
    pub key: Key,
    pub size: i64,
}

bitflags! {
    pub struct Include : u32 {
        const NOTHING = 0x00;
        const DIRS = 0x01;
        const FILES = 0x02;
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[trait_async]
pub trait Store: Send + Sync {
    async fn list_contents(&self, path: &str, flags: Include) -> Result<Vec<ObjectInfo>>;

    async fn get(&self, key: Key) -> Result<Vec<u8>>;
}
