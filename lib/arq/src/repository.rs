use arq_storage::{Error as StorageError, Include, Key as StorageKey, Store};

use futures::future;
use log::info;
use serde::Deserialize;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    store: Arc<dyn Store>,
}

#[derive(Deserialize, Debug)]
pub struct Computer {
    #[serde(skip)]
    pub id: Uuid,

    #[serde(rename = "userName")]
    pub user: String,

    #[serde(rename = "computerName")]
    pub computer: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Storage(StorageError),
    MalformedData,
}

async fn fetch_computer(
    store: &dyn Store,
    id: Uuid,
    key: StorageKey,
) -> Result<Computer, RepoError> {
    // fetch and parse the computerinfo file
    let content = store
        .get(key / "computerinfo")
        .await
        .map_err(RepoError::Storage)?;

    plist::from_reader(Cursor::new(content))
        .map(|cmp| Computer { id, ..cmp })
        .map_err(|_| RepoError::MalformedData)
}

impl Repository {
    pub fn new(store: Arc<dyn Store>) -> Repository {
        Repository { store }
    }

    pub async fn salt(&self) -> Result<Vec<u8>, RepoError> {
        Err(RepoError::MalformedData)
        //        self.transport.get(self.root_prefix.clone() / "salt")
    }

    pub async fn list_computers(&self) -> Result<Vec<Computer>, RepoError> {
        let folders = self
            .store
            .list_contents("", Include::DIRS)
            .await
            .map_err(RepoError::Storage)?;

        // build a list of items to pull from the store and wrap them in
        // futures that do the work of pulling them down and parsing them
        // into computer info
        let tasks: Vec<_> = folders
            .iter()
            .filter_map(|d| {
                // remove trailing delimiter & attempt to parse as a
                // UUID. Unsucesful attempts are filtered out of the
                // result set
                let s = d.key.as_str();
                let key = &s[0..s.len() - 1];
                Uuid::parse_str(key)
                    .map(|id| (id, StorageKey::from(key)))
                    .ok()
            })
            .map(|(id, computer_key)| fetch_computer(self.store.as_ref(), id, computer_key))
            .collect();

        // Run all the fetches in parallel and filter out all the items
        // that failed
        let result = future::join_all(tasks)
            .await
            .drain(..)
            .filter_map(|x| x.ok())
            .collect();
        Ok(result)
    }

    pub async fn list_folders(&self, computer_id: &Uuid) -> Result<Vec<StorageKey>, ()> {
        let computer_root = computer_id
            .to_hyphenated_ref()
            .encode_upper(&mut Uuid::encode_buffer())
            .to_owned();
        let path = format!("{}/buckets/", computer_root);
        let folders = self
            .store
            .list_contents(&path, Include::DIRS)
            .await
            .map_err(|_| ())?;

        Ok(folders.iter().map(|obj| obj.key.clone()).collect())
    }
}
