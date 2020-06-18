use crate::encryption::{CryptoKey, Decrypter};
use arq_storage::{Error as StorageError, Include, Key as StorageKey, Store};

use futures::future;
use serde::Deserialize;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;
use log::{error, info};
/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    store: Arc<dyn Store>,
}

#[derive(Deserialize, Debug)]
pub struct Computer {
    #[serde(skip)]
    pub id: String,

    #[serde(skip)]
    pub salt: Vec<u8>,

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

async fn fetch_computer(store: &dyn Store, id: String) -> Result<Computer, RepoError> {
    let machine_key = StorageKey::from(id);
    let (info, salt) = future::try_join(
        store.get(&machine_key / "computerinfo"),
        store.get(&machine_key / "salt"),
    )
    .await
    .map_err(RepoError::Storage)?;

    plist::from_reader(Cursor::new(info))
        .map(|cmp| Computer {
            id: machine_key.into_string(),
            salt,
            ..cmp
        })
        .map_err(|_| RepoError::MalformedData)
}

#[derive(Debug)]
pub struct Folder {}

async fn fetch_folder(
    store: &dyn Store,
    key: StorageKey,
    decrypter: &dyn Decrypter,
) -> Result<Folder, RepoError> {
    info!("Fetching {:?}", key);
    // TODO - examine how to do a streaming decrypt, rather than a one-hit
    // buffered decrypt
    let encrypted_object = store.get(key).await.map_err(RepoError::Storage)?;

    let encrypted_bytes = &encrypted_object[9..];

    info!("decrypting {} bytes", encrypted_bytes.len());
    let r = decrypter.decrypt(encrypted_bytes);
    if let Ok(b) = r {
        info!("Ka-ching: {} bytes", b.len());
        info!("text: {}", std::str::from_utf8(&b[..]).unwrap());
    }

    Ok(Folder {})
}

impl Repository {
    pub fn new(store: Arc<dyn Store>) -> Repository {
        Repository { store }
    }

    pub async fn get_computer(&self, id: &str) -> Result<Computer, RepoError> {
        fetch_computer(self.store.as_ref(), id.to_owned()).await
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
            .into_iter()
            .filter_map(|d| {
                // remove trailing delimiter & attempt to parse as a
                // UUID. Unsucesful attempts are filtered out of the
                // result set
                let s = d.key.as_str();
                let key = &s[0..s.len() - 1];

                // if it parses as a UUID, we want to return the key
                // as a *string* - nobody upstream cares that its a UUID.
                Uuid::parse_str(key).map(|_| key.to_owned()).ok()
            })
            .map(|computer_key| fetch_computer(self.store.as_ref(), computer_key))
            .collect();

        // Run all the fetches in parallel and filter out all the items
        // that failed
        let result = future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(|x| x.ok())
            .collect();
        Ok(result)
    }

    pub async fn list_folders(
        &self,
        computer_id: &str,
        decrypter: &dyn Decrypter,
    ) -> Result<Vec<Folder>, RepoError> {
        info!("Listing folders...");
        let path = format!("{}/buckets/", computer_id);
        let folder_buckets = self
            .store
            .list_contents(&path, Include::FILES)
            .await
            .map_err(RepoError::Storage)?;


        info!("Building task list");
        let tasks: Vec<_> = folder_buckets
            .into_iter()
            .map(|obj| fetch_folder(self.store.as_ref(), obj.key, decrypter))
            .collect();

        info!("Spawning {} subtasks", tasks.len());
        let folders = future::join_all(tasks).await;

        info!("Collating resuts");
        let mut result = Vec::with_capacity(folders.len());
        for maybe_folder in folders.into_iter() {
            if let Err(e) = maybe_folder {
                error!("Kaboom: {:?}", e);
                return Err(RepoError::MalformedData);
            }
            result.push(maybe_folder.unwrap());
        }

        Ok(result)
    }
}
