mod key;
mod object_decrypter;

pub use key::CryptoKey;
pub use object_decrypter::ObjectDecrypterV1;

pub enum CryptoError {
    BadKey,
    MalformedData,
    Unexpected,
}

pub trait ObjectDecrypter {
    fn decrypt_object(&self, object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
