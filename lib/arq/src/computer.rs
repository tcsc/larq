use arq_crypto::ObjectDecrypter;
use arq_storage::{Include, Store};
use futures::{future, TryFutureExt};
use log::{debug, error, info};
use serde::Deserialize;
use std::{fmt, sync::Arc};

use crate::{storage::Key as StorageKey, Folder, FolderInfo, RepoError};

#[derive(Deserialize, Debug)]
pub struct ComputerInfo {
    #[serde(skip)]
    pub id: String,

    #[serde(rename = "userName")]
    pub user: String,

    #[serde(rename = "computerName")]
    pub computer: String,
}

pub struct Computer {
    info: ComputerInfo,
    store: Arc<dyn Store>,
    decrypter: Arc<dyn ObjectDecrypter>,
    bucket_decrypter: Arc<dyn ObjectDecrypter>,
}

impl fmt::Debug for Computer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Computer")
            .field("info", &self.info)
            .finish()
    }
}

impl Computer {
    pub fn new(
        info: ComputerInfo,
        decrypter: &Arc<dyn ObjectDecrypter>,
        bucket_decrypter: &Arc<dyn ObjectDecrypter>,
        store: &Arc<dyn Store>,
    ) -> Computer {
        Computer {
            info,
            store: store.clone(),
            decrypter: decrypter.clone(),
            bucket_decrypter: bucket_decrypter.clone(),
        }
    }

    pub async fn list_folders(&self) -> Result<Vec<crate::FolderInfo>, crate::RepoError> {
        info!("Listing folders...");
        let path = format!("{}/buckets/", self.info.id);
        let folder_buckets = self
            .store
            .list_contents(&path, Include::FILES)
            .await
            .map_err(RepoError::Storage)?;

        debug!("Building task list");
        let tasks: Vec<_> = folder_buckets
            .into_iter()
            .map(|obj| fetch_folder(self.store.as_ref(), obj.key, self.bucket_decrypter.as_ref()))
            .collect();

        debug!("Spawning {} subtasks", tasks.len());
        let folders = future::join_all(tasks).await;

        debug!("Collating resuts");
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

    pub async fn get_folder(&self, folder_id: &str) -> Result<Folder, RepoError> {
        let key = StorageKey::from(format!("{}/buckets/{}", self.info.id, folder_id));
        fetch_folder(
            self.store.as_ref(),
            key.clone(),
            self.bucket_decrypter.as_ref(),
        )
        .and_then(|info| Folder::new(&self.info.id, info, &self.store, &self.decrypter))
        .await
    }
}

const V1_HEADER: &[u8] = "encrypted".as_bytes();

async fn fetch_folder(
    store: &dyn Store,
    key: StorageKey,
    decrypter: &dyn ObjectDecrypter,
) -> Result<FolderInfo, RepoError> {
    debug!("Fetching {:?}", key);
    // TODO - examine how to do a streaming decrypt, rather than a one-hit
    // buffered decrypt
    let encrypted_object = store.get(key).await.map_err(RepoError::Storage)?;

    if encrypted_object.len() < V1_HEADER.len() || &encrypted_object[..9] != V1_HEADER {
        return Err(RepoError::MalformedData);
    }

    debug!("decrypting {}-byte object", encrypted_object.len());
    let obj = decrypter
        .decrypt_object(&encrypted_object[V1_HEADER.len()..])
        .map_err(|_| RepoError::CryptoError)?;

    drop(encrypted_object);

    plist::from_bytes(&obj[..]).map_err(|_| RepoError::MalformedData)
}
