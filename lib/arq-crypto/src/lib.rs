pub mod key;

pub use key::CryptoKey;

pub trait Encrypter {
    fn encrypt(&self, buf: &[u8]) -> Result<Vec<u8>, ()>;
}

pub trait Decrypter {
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>, ()>;
}

pub enum CryptoError {
    BadKey,
    MalformedData,
}

pub trait ObjectDecrypter {
    fn decrypt_object(&self, object_bytes: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
