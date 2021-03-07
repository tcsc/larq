mod commit;
mod compression;
mod computer;
mod constructs;
mod folder;
mod packset;
mod repository;
mod sha;
mod tree;

pub mod storage {
    pub use arq_storage::*;
}

pub mod s3 {
    pub use arq_s3::Store;
}

pub mod crypto {
    pub use arq_crypto::*;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Storage(arq_storage::Error),
    MalformedData,
    CryptoError, // probably bad key
}

pub use computer::{Computer, ComputerInfo};
pub use folder::{Folder, FolderInfo};
pub use packset::Packset;
pub use repository::Repository;
pub use sha::SHA1;

pub fn format_uuid(id: &uuid::Uuid) -> String {
    let mut buf = uuid::Uuid::encode_buffer();
    id.to_hyphenated_ref().encode_upper(&mut buf).to_owned()
}
