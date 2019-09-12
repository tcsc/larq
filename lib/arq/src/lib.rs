mod key;
mod repository;
pub mod s3;
//mod encryption;

pub use key::Key;
pub use repository::Repository;

use bitflags::bitflags;
use futures::Future;

#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
    NoSuchObject,
    AccessDenied,
    NetworkError,
    UnknownError,
}

pub type StorageFuture<T> = Box<dyn Future<Item = T, Error = StorageError> + Send>;

/**
 * An abstract storage object managed by a storage service.
 */
pub struct StorageObject {
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

pub trait Store {
    fn list_contents(&self, path: &str, flags: Include) -> StorageFuture<Vec<StorageObject>>;

    fn get(&self, key: Key) -> StorageFuture<Vec<u8>>;
}
