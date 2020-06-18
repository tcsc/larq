use crate::{CryptoError, CryptoKey, ObjectDecrypter};

pub struct ObjectDecrypterV1 {
    key: CryptoKey,
}

impl ObjectDecrypterV1 {
    pub fn new(key: CryptoKey) -> Self {
        ObjectDecrypterV1 { key: key }
    }
}

const V1_HEADER: &'static [u8] = "encrypted".as_bytes();

impl ObjectDecrypter for ObjectDecrypterV1 {
    fn decrypt_object(&self, object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if (object_bytes.len() < V1_HEADER.len()) || (&object_bytes[..9] != V1_HEADER) {
            return Err(CryptoError::MalformedData);
        }

        self.key.decrypt(&object_bytes[V1_HEADER.len()..])
    }
}
