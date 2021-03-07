use std::convert::TryFrom;

use crate::{commit::CompressionType, SHA1};

use chrono::prelude::*;

#[derive(Debug)]
pub enum StorageType {
    None,
    S3,
    Glacier,
}

impl TryFrom<u32> for StorageType {
    type Error = String;

    fn try_from(n: u32) -> Result<StorageType, String> {
        match n {
            0 => Ok(StorageType::None),
            1 => Ok(StorageType::S3),
            2 => Ok(StorageType::Glacier),
            _ => {
                let msg = format!("Invalid storage type: {:x}", n);
                Err(msg)
            }
        }
    }
}

#[derive(Debug)]
pub struct BlobKey {
    pub sha: SHA1,
    pub stretch_key: bool,
    pub storage_type: StorageType,
    pub size: Option<u64>,
    pub upload_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub is_tree: bool,
    pub has_missing_items: Option<bool>,
    pub data_compression_type: CompressionType,
    pub data_blob_keys: Vec<BlobKey>,
    pub data_size: u64,
    pub xattrs_compression_type: CompressionType,
    pub xattrs_blob_key: Option<BlobKey>,
    pub xattrs_size: u64,
    pub acl_compression_type: CompressionType,
    pub acl_blob_key: Option<BlobKey>,
    pub user_id: i32,
    pub group_id: i32,
    pub file_mode: i32,
    pub flags: u64,
    pub finder_flags: u64,
    pub mod_time: DateTime<Utc>,
    pub c_time: DateTime<Utc>,
    pub create_time: DateTime<Utc>,
    pub file_type: Option<String>,
    pub creator: Option<String>,
    pub hide_extension: bool,
    pub st_dev: i32,
    pub st_ino: i32,
    pub st_nlink: u32,
    pub st_rdev: i32,
    pub st_blocks: i64,
    pub st_block_size: i32,
}

#[derive(Debug)]
pub struct Tree {
    pub version: usize,
    pub xattrs_compression_type: CompressionType,
    pub acl_compression_type: CompressionType,
    pub xattrs_blob_key: Option<BlobKey>,
    pub xattrs_blob_size: u64,
    pub acl_blob_key: Option<BlobKey>,
    pub user_id: i32,
    pub group_id: i32,
    pub file_mode: i32,
    pub mod_time: DateTime<Utc>,
    pub flags: u64,
    pub finder_flags: u64,
    pub st_dev: i32,
    pub st_ino: i32,
    pub st_nlink: u32,
    pub st_rdev: i32,
    pub c_time: DateTime<Utc>,
    pub st_blocks: i64,
    pub st_block_size: usize,
    pub size_on_disk: u64,
    pub creation_time: DateTime<Utc>,
    pub missing_nodes: Vec<String>,
    pub nodes: Vec<Node>,
}
