use crate::{CryptoError, CryptoKey, ObjectDecrypter};

#[derive(Clone)]
pub struct ObjectDecrypterV1 {
    key: CryptoKey,
}

impl ObjectDecrypterV1 {
    pub fn new(key: CryptoKey) -> Self {
        ObjectDecrypterV1 { key }
    }
}

impl ObjectDecrypter for ObjectDecrypterV1 {
    fn decrypt_object(&self, object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.key.decrypt(object_bytes)
    }
}
