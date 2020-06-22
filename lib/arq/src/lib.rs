mod computer;
mod folder;
mod repository;

pub mod s3 {
    pub use arq_s3::Store;
}

pub mod crypto {
    pub use arq_crypto::{CryptoKey, ObjectDecrypterV1};
}

pub use arq_storage::{Error as StorageError, Key as StorageKey};

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Storage(arq_storage::Error),
    MalformedData,
    CryptoError, // probably bad key
}

pub use computer::Computer;
pub use folder::Folder;
pub use repository::Repository;

pub fn format_uuid(id: &uuid::Uuid) -> String {
    let mut buf = uuid::Uuid::encode_buffer();
    id.to_hyphenated_ref().encode_upper(&mut buf).to_owned()
}
