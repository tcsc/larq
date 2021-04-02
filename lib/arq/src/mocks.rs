use async_trait::async_trait;

use crate::{
    crypto::{CryptoError, ObjectDecrypter},
    storage::{self, Include, Key, ObjectInfo, Store},
};

struct NullStore {}

#[async_trait]
impl Store for NullStore {
    async fn list_contents(&self, _path: &str, _flags: Include) -> storage::Result<Vec<ObjectInfo>> {
        Ok(Vec::new())
    }

    async fn get(&self, _key: Key) -> storage::Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

struct NullDecrypter {}

impl ObjectDecrypter for NullDecrypter {
    fn decrypt_object(&self, _object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Ok(Vec::new())
    }
}
