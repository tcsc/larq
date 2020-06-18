pub mod key;

pub trait Encrypter {
    fn encrypt(&self, buf: &[u8]) -> Result<Vec<u8>, ()>;
}

pub trait Decrypter {
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>, ()>;
}


pub trait ObjectDecrypter {
    
}

pub use key::*;
