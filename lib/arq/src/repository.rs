use arq_storage::{Error as StorageError, Include, Key as StorageKey, Store};
use futures::future;
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
            .iter()
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
            .drain(..)
            .filter_map(|x| x.ok())
            .collect();
        Ok(result)
    }

    pub async fn list_folders(&self, computer_id: &str) -> Result<Vec<StorageKey>, ()> {
        let path = format!("{}/buckets/", computer_id);
        let folders = self
            .store
            .list_contents(&path, Include::FILES)
            .await
            .map_err(|_| ())?;

        Ok(folders.iter().map(|obj| obj.key.clone()).collect())
    }
}
