mod record;

use std::{
    path::{self, PathBuf},
    sync::Arc,
};

use chrono::prelude::*;
use glob::Pattern;
use log::{error, info};

use crate::{
    compression::decompress,
    crypto::ObjectDecrypter,
    storage::Store,
    tree::{self, BlobKey, StorageType},
    CompressionType, Packset, RepoError,
};

use record::CommitRecord;

pub struct Commit<'a> {
    record: CommitRecord,
    packset: &'a Packset,
    store: Arc<dyn Store>,
    decrypter: Arc<dyn ObjectDecrypter>,
}

impl<'a> Commit<'a> {
    pub fn parse(
        blob: &[u8],
        packset: &'a Packset,
        decrypter: &Arc<dyn ObjectDecrypter>,
    ) -> Result<Self, RepoError> {
        let record = record::parse(blob)?;
        Ok(Commit {
            record,
            packset,
            store: packset.store().clone(),
            decrypter: decrypter.clone(),
        })
    }

    pub async fn list_files(&self, pattern: &str) -> Result<(), RepoError> {
        let commit = &self.record;
        let patterns = parse_pattern(pattern)?;

        struct Child {
            pattern_index: isize,
            keys: Vec<BlobKey>,
            path: PathBuf,
            compression_type: CompressionType,
        };

        let root = Child {
            pattern_index: -1,
            path: PathBuf::new(),
            keys: vec![BlobKey {
                sha: commit.tree_sha.clone(),
                stretch_key: commit.expand_key,
                storage_type: StorageType::S3,
                size: None,
                upload_date: None,
            }],
            compression_type: commit.compression_type,
        };

        let mut pending_children = vec![root];

        while let Some(j) = pending_children.pop() {
            info!("Loading child {:?}", j.keys);
            let t = load_blob(
                &self.packset,
                &j.keys,
                self.decrypter.as_ref(),
                j.compression_type,
            )
            .await
            .and_then(|d| {
                use std::io::Write;
                let mut f = std::fs::File::create("child.blob").expect("file");
                f.write_all(&d);
                drop(f);

                tree::parse(&d)
            })
            .map_err(|e| {
                error!("Parsing tree failed: {:?}", e);
                e
            })?;

            for n in t.nodes {
                if n.is_tree {
                    let child = Child {
                        pattern_index: 0, // not used yet
                        path: j.path.join(n.name),
                        keys: n.data_blob_keys,
                        compression_type: n.data_compression_type,
                    };
                    pending_children.push(child);
                    continue;
                }

                println!("{:?}: {} bytes", j.path.join(n.name), n.data_size);
            }
        }

        Ok(())
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.record.timestamp
    }
}

async fn load_blob(
    packset: &Packset,
    keys: &[BlobKey],
    decrypter: &dyn ObjectDecrypter,
    compression_type: CompressionType,
) -> Result<Vec<u8>, RepoError> {
    let fetch_tasks = keys
        .iter()
        .map(|k| load_blob_fragment(packset, k, decrypter, compression_type));
    let blobs = futures::future::try_join_all(fetch_tasks).await?;
    let overall_len = blobs.iter().fold(0, |acc, x| acc + x.len());
    let mut result = Vec::with_capacity(overall_len);
    for mut b in blobs.into_iter() {
        result.append(&mut b);
    }

    Ok(result)
}

async fn load_blob_fragment(
    packset: &Packset,
    key: &BlobKey,
    decrypter: &dyn ObjectDecrypter,
    compression_type: CompressionType,
) -> Result<Vec<u8>, RepoError> {
    let encrypted_object = packset.load(&key.sha).await?;

    let decrypted_object = decrypter
        .decrypt_object(&encrypted_object.content)
        .map_err(|_| RepoError::CryptoError)?;
    drop(encrypted_object);

    if compression_type == CompressionType::None {
        Ok(decrypted_object)
    } else {
        decompress(&decrypted_object, compression_type)
    }
}

fn parse_pattern(pattern_text: &str) -> Result<Vec<glob::Pattern>, RepoError> {
    let components = pattern_text.split_terminator(std::path::is_separator);
    let mut result = Vec::new();
    for component in components {
        let p = Pattern::new(component).map_err(|_| RepoError::InputError)?;
        result.push(p);
    }
    Ok(result)
}
