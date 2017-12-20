#[macro_use] extern crate bitflags;
extern crate chrono;
extern crate hyper;
extern crate rusoto_core;
extern crate rusoto_s3;

#[macro_use] extern crate log;

pub mod s3;
mod key;

pub use key::Key;

#[derive(Debug, Eq, PartialEq) ]
pub enum TransportError {
    NoSuchObject,
    AccessDenied,
    NetworkError,
    UnknownError
}

/**
 * An abstract storage object managed by a storage service.
 */
pub struct StorageObject {
    pub key: Key,
    pub size: i64
}

bitflags!{
    pub struct Include : u32 {
        const NOTHING = 0x00;
        const DIRS = 0x01;
        const FILES = 0x02;
    }
}

pub trait Store {
    fn list_contents(&self, path: &str, flags: Include) -> Result<Vec<StorageObject>, TransportError>;
    fn get(&self, key: Key) -> Result<Vec<u8>, TransportError>;
}


/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    root_prefix: Key,
    transport: Box<Store>,
}

impl Repository {
    pub fn new(computer_id: &str, transport: Box<Store>) -> Repository {
        Repository {
            root_prefix: Key::from(computer_id),
            transport
        }
    }

    pub fn salt(&self) -> Result<Vec<u8>, TransportError> {
        self.transport.get(self.root_prefix.clone() / "salt")
    }

    pub fn list_backup_sets(&self) {

    }
}