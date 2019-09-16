use super::{Include, StorageError, Store};
use crate::gather::gather_all;
use crate::key::Key;
use futures::{future, Future};
use log::info;
use serde::Deserialize;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    root_prefix: Key,
    transport: Arc<dyn Store>,
}

#[derive(Deserialize, Debug)]
pub struct Computer {
    #[serde(rename = "userName")]
    pub user: String,
    #[serde(rename = "computerName")]
    pub computer: String,
}

impl Repository {
    pub fn new(computer_id: &str, transport: Arc<dyn Store>) -> Repository {
        let root_prefix = Key::from(computer_id);
        //let salt = transport.get(root_prefix.clone() / "salt");
        Repository {
            root_prefix,
            transport,
        }
    }

    pub fn salt(&self) -> impl Future<Item = Vec<u8>, Error = StorageError> {
        future::err(StorageError::UnknownError)
        //        self.transport.get(self.root_prefix.clone() / "salt")
    }

    pub fn list_computers(&self) -> impl Future<Item = Vec<Computer>, Error = StorageError> {
        let t = self.transport.clone();
        self.transport
            .list_contents("", Include::DIRS)
            .and_then(move |folders| {
                let tasks: Vec<_> = folders
                    .iter()
                    .filter_map(|d| {
                        // remove trailing delimiter & attempt to parse as a
                        // UUID. Unsucesful attempts are filtered out of the
                        // result set
                        let s = d.key.as_str();
                        let key = &s[0..s.len() - 1];
                        Uuid::parse_str(key).map(|id| (id, Key::from(key))).ok()
                    })
                    .map(move |(id, computer_key)| {
                        // fetch and parse the computerinfo file
                        t.get(computer_key / "computerinfo").and_then(|content| {
                            let c = Cursor::new(content);
                            match plist::from_reader(c) {
                                Ok(computer) => future::ok(computer),
                                Err(_) => future::err(StorageError::UnknownError),
                            }
                        })
                    })
                    .collect();

                // run all the tasks to completion and filter out all the errors
                gather_all(tasks).map(|ts| {
                    ts.into_iter()
                        .filter_map(|x| -> Option<Computer> { x.ok() })
                        .collect()
                })
            })
    }

    pub fn list_backup_sets(&self) {}
}
