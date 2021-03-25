use futures::future;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    computer::{Computer, ComputerInfo},
    RepoError,
};
use arq_crypto::{CryptoKey, ObjectDecrypter, ObjectDecrypterV1};
use arq_storage::{Include, Key as StorageKey, Store};

/**
 * Wraps up access to a backup repository
 */
pub struct Repository {
    store: Arc<dyn Store>,
    secret: String,
}

async fn fetch_computer_info(store: &dyn Store, id: StorageKey) -> Result<ComputerInfo, RepoError> {
    let info = store
        .get(&id / "computerinfo")
        .await
        .map_err(RepoError::Storage)?;

    plist::from_bytes(&info[..])
        .map(|cmp| ComputerInfo {
            id: id.to_string(),
            ..cmp
        })
        .map_err(|_| RepoError::MalformedData)
}

impl Repository {
    pub fn new(secret: &str, store: Arc<dyn Store>) -> Repository {
        Repository {
            secret: secret.to_owned(),
            store,
        }
    }

    pub async fn get_computer(&self, id: String) -> Result<Computer, RepoError> {
        let machine_key = StorageKey::from(id);

        let salt = self
            .store
            .get(machine_key.clone() / "salt")
            .await
            .map_err(RepoError::Storage)?;

        let object_decrypter = CryptoKey::new(&self.secret, &salt[..])
            .map(ObjectDecrypterV1::new)
            .map(|d| Arc::new(d) as Arc<dyn ObjectDecrypter>)
            .map_err(|_| RepoError::CryptoError)?;

        // if repo version == 1, otherwise re-use object decrypter
        let bucket_decrypter = CryptoKey::new(&self.secret, "BucketPL".as_bytes())
            .map(ObjectDecrypterV1::new)
            .map(|d| Arc::new(d) as Arc<dyn ObjectDecrypter>)
            .map_err(|_| RepoError::CryptoError)?;

        let info = fetch_computer_info(self.store.as_ref(), machine_key.clone()).await?;

        Ok(Computer::new(
            info,
            &object_decrypter,
            &bucket_decrypter,
            &self.store,
        ))
    }

    // pub async fn get_computer(&self, id: &str) -> Result<Computer, RepoError> {
    //     fetch_computer(self.store.as_ref(), id.to_owned()).await
    // }

    pub async fn list_computers(&self) -> Result<Vec<ComputerInfo>, RepoError> {
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
                Uuid::parse_str(key).map(|_| StorageKey::from(key)).ok()
            })
            .map(|computer_key| fetch_computer_info(self.store.as_ref(), computer_key))
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
}
