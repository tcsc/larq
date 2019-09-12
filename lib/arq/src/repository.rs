use super::{StorageError, Store};
use crate::key::Key;
use futures::Future;

/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    root_prefix: Key,
    transport: Box<dyn Store>,
}

impl Repository {
    pub fn new(computer_id: &str, transport: Box<dyn Store>) -> Repository {
        let root_prefix = Key::from(computer_id);
        //let salt = transport.get(root_prefix.clone() / "salt");
        Repository {
            root_prefix,
            transport,
        }
    }

    pub fn salt(&self) -> impl Future<Item=Vec<u8>, Error=StorageError> {
        self.transport.get(self.root_prefix.clone() / "salt")
    }

    pub fn list_backup_sets(&self) {}
}
