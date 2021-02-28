use std::sync::Arc;

use crate::{
    format_uuid,
    storage::{Key, Store},
    RepoError, SHA1,
};

mod index;
use index::PackIndex;

pub struct Packset {
    root: Key,
    index: PackIndex,
    store: Arc<dyn Store>,
}

// TODO: data file caching

impl Packset {
    pub async fn new(key: Key, store: &Arc<dyn Store>) -> Result<Self, RepoError> {
        index::load(&key, store.as_ref()).await.map(|i| Packset {
            root: key,
            index: i,
            store: store.clone(),
        })
    }

    pub async fn get(&self, id: &SHA1) -> Result<Vec<u8>, RepoError> {
        let loc = self.index.get(id).ok_or(RepoError::MalformedData)?;
        let packfile_key = (&self.root) / &(hex::encode(&loc.pack_id) + ".data");
        let packfile_data = self.store.get(packfile_key).await;
        unimplemented!();
    }
}
