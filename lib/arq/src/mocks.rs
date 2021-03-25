use async_trait::async_trait;

use crate::{
    storage::{self, Key, Store, Include, ObjectInfo},
    crypto::{ObjectDecrypter, CryptoError}
};

struct NullStore {}

#[async_trait]
impl Store for NullStore {
    async fn list_contents(&self, path: &str, flags: Include) -> storage::Result<Vec<ObjectInfo>> {
        Ok(Vec::new())
    }

    async fn get(&self, key: Key) -> storage::Result<Vec<u8>> {
        Ok(Vec::new())
    }
}


struct NullDecrypter {}

impl ObjectDecrypter for NullDecrypter {
    fn decrypt_object(&self, object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Ok(Vec::new())
    }
}
