use std::sync::Arc;

use crate::{
    storage::{Key, Store},
    RepoError, SHA1,
};

mod index;
mod pack;

use index::PackIndex;
pub use pack::PackedObject;

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

    pub async fn load(&self, id: &SHA1) -> Result<PackedObject, RepoError> {
        let loc = self.index.get(id).ok_or(RepoError::MalformedData)?;

        log::info!(
            "Blob is in pack {}, {} bytes from offset {}",
            loc.pack_id,
            loc.length,
            loc.offset
        );

        let packfile_key = (&self.root) / &(loc.pack_id.as_string() + ".pack");
        log::info!("Fetching blob {}", packfile_key.as_str());
        let packfile_data = self
            .store
            .get(packfile_key)
            .await
            .map_err(RepoError::Storage)?;

        // Perhaps verify that this is a pack file here?
        log::info!("Extracting blob...");
        let start = loc.offset as usize;
        pack::parse_object(&packfile_data[start..])
    }
}
