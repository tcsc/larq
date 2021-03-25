// use std::sync::Arc;

// use crate::{
//     commit::{self, Commit, CompressionType},
//     compression::decompress,
//     crypto::ObjectDecrypter, 
//     packset::Packset,
//     tree::{self, Tree},
//     RepoError,
//     SHA1
// };

// pub struct TreeIndex {
//     packset: Packset,
//     decrypter: Arc<dyn ObjectDecrypter>,
// }

// impl TreeIndex {
//     pub fn new(packset: Packset, decrypter: &Arc<dyn ObjectDecrypter>) -> TreeIndex {
//         TreeIndex {
//             packset: packset,
//             decrypter: decrypter.clone(),
//         }
//     }

//     pub async fn load(&self, commit_id: &SHA1) -> Result<Commit, RepoError> {
//         log::info!("Loading commit {}", commit_id);
//         let commit = 
//             self.packset.load(&commit_id)
//                 .await
//                 .and_then(|blob| {
//                     self.decrypter
//                         .decrypt_object(&blob.content)
//                         .map_err(|e| RepoError::CryptoError)
//                         .and_then(|d| Commit::parse(&d, &self))  
//                 })?;
 
//         log::info!("Loading tree from {} (expand key: {}, compression: {:?})", 
//             commit.tree_sha, 
//             commit.expand_key,
//             commit.compression_type);
//         let _tree = 
//             self.packset.load(&commit.tree_sha)
//                 .await
//                 .and_then(|blob| {
//                     self.decrypter
//                         .decrypt_object(&blob.content)
//                         .map_err(|_| RepoError::CryptoError)
//                         .and_then(|d| {
//                             if commit.compression_type == CompressionType::None {
//                                 d
//                             } else {
//                                 log::info!("Decompressing blob..");
//                                 decompress(&d, commit.compression_type)
//                             }
//                         })
//                         .and_then(|d| tree::parse(&d))
//                 })?;

//         unimplemented!()

//     //     let tree =  
//     //         self.packset.load(commit.tree_sha)
//     //             .await
//     //             .and_then(|blob| {
//     //                 self.decrypter
//     //                     .decrypt_object(&encrypted_blob.content)
//     //                     .map_err(|e| RepoError::CryptoError)
//     //                     .and_then(|d| commit::parse(&d[..]))?;    
//     //             })
//     }
// }
