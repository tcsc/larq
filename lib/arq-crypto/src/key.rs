// use std::num::NonZeroU32;
//
// use ring::{
//     digest,
//     pbkdf2::{self, PBKDF2_HMAC_SHA1},
// };

use crate::CryptoError;
use std::vec::Vec;

const KEY_LEN: usize = 48;
const KEY_ITER: usize = 1000;

use openssl::{
    hash::MessageDigest,
    pkcs5,
    symm::{decrypt, encrypt, Cipher},
};

#[derive(Clone)]
pub struct CryptoKey {
    cipher: Cipher,
    key: Vec<u8>,
    iv: Option<Vec<u8>>,
}

impl CryptoKey {
    pub fn new(secret: &str, salt: &[u8]) -> Result<CryptoKey, CryptoError> {
        let mut key_bytes: [u8; KEY_LEN] = [0; KEY_LEN];

        pkcs5::pbkdf2_hmac(
            secret.as_bytes(),
            salt,
            KEY_ITER,
            MessageDigest::sha1(),
            &mut key_bytes[..],
        )
        .map_err(CryptoError::LibraryError)?;

        let cipher = Cipher::aes_256_cbc();

        pkcs5::bytes_to_key(
            cipher,
            MessageDigest::sha1(),
            &key_bytes[..],
            Some(salt),
            KEY_ITER as i32,
        )
        .map_err(|_| CryptoError::Unexpected)
        .map(|k| CryptoKey {
            key: k.key,
            iv: k.iv,
            cipher,
        })
    }

    pub fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let iv = self.iv.as_deref();
        decrypt(self.cipher, &self.key[..], iv, buf).map_err(|_| CryptoError::BadKey)
    }

    pub fn encrypt(&self, buf: &[u8]) -> Result<Vec<u8>, ()> {
        let iv = self.iv.as_deref();
        encrypt(self.cipher, &self.key[..], iv, buf).map_err(|_| ())
    }
}
