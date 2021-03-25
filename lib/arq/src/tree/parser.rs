use std::convert::TryFrom;

use crate::{
    constructs::*,
    tree::{BlobKey, Node, StorageType, Tree},
    RepoError,
};

use nom::{
    call,
    cond,
    do_parse,
    many_m_n,
    map,
    map_res,
    named,
    named_args,
    // dbg_dmp,
    number::streaming::{be_i32, be_i64, be_u32, be_u64},
};

use chrono::prelude::*;

named!(
    storage_type<StorageType>,
    map_res!(be_u32, TryFrom::<u32>::try_from)
);

// NULL Blobkeys are written as a string of zero values, rather than having a
// "present" flag at the start. I assume that this is due to a bug in Arq from
// a decade ago that we're stuck with.
named_args!(
    maybe_blob_key(tree_version: usize)<Option<BlobKey>>,
    do_parse!(
        maybe_sha: maybe_sha_string
        >> expand: cond!(tree_version >= 14, boolean)
        >> storage_type: cond!(tree_version >= 17, storage_type)
        >> archive_id: cond!(tree_version >= 17, maybe_string)
        >> size: cond!(tree_version >= 17, be_u64)
        >> date: cond!(tree_version >= 17, maybe_date_time)
        >> ( maybe_sha.map(|sha| {
                BlobKey {
                    sha,
                    stretch_key: expand.unwrap_or(false),
                    storage_type: storage_type.unwrap_or(StorageType::S3),
                    size,
                    upload_date: date.unwrap_or(None),
                }
            })
        )
    )
);

named_args!(
    blob_key(version: usize)<BlobKey>,
    map_res!(call!(maybe_blob_key, version), |b: Option<BlobKey>| {
        b.ok_or("Blob key may not be null")
    })
);

fn as_datetime(s: i64, ns: i64) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(s, ns as u32), Utc)
}

named!(
    missing_nodes<Vec<String>>,
    do_parse!(n: map!(be_u32, |x| x as usize) >> v: many_m_n!(n, n, non_null_string) >> (v))
);

named_args!(
    blob_keys(version: usize)<Vec<BlobKey>>,
    do_parse!(
        n: map!(be_u32, |x| x as usize)
        >> v: many_m_n!(n, n, call!(blob_key, version))
        >> (v))
);

named_args!(
    node(version: usize)<Node>,
    do_parse!(
        name: non_null_string
        >> is_tree: boolean
        >> has_missing_items: cond!(version >= 18, boolean)
        >> data_is_compressed: cond!((12..=18).contains(&version), boolean)
        >> data_compression_type: cond!(version >= 19, compression_type)
        >> xattrs_are_compressed: cond!((12..=18).contains(&version), boolean)
        >> xattrs_compression_type: cond!(version >= 19, compression_type)
        >> acl_is_compressed: cond!((12..=18).contains(&version), boolean)
        >> acl_compression_type: cond!(version >= 19, compression_type)
        >> blob_keys: call!(blob_keys, version)
        >> data_size: be_u64
        // NB: The docs say <= v18, but the arq restore implemetation says a strictly
        // less than, and the actual v18 blobs agree
        >> thumbnail_sha: cond!(version < 18, maybe_sha_string)
        >> stretch_thumbnail_key: cond!((14..=17).contains(&version), boolean)
        >> preview_sha: cond!(version < 18, maybe_sha_string)
        >> stretch_preview_key: cond!((14..=17).contains(&version), boolean)

        >> xattrs_blob_key: call!(maybe_blob_key, version)
        >> xattrs_size: be_u64
        >> acl_blob_key: call!(maybe_blob_key, version)
        >> user_id: be_i32
        >> group_id: be_i32
        >> file_mode: be_i32
        >> mtime_sec: be_i64
        >> mtime_nsec: be_i64
        >> flags: be_u64
        >> finder_flags: be_u32
        >> extended_finder_flags: be_u32
        >> file_type: maybe_string
        >> file_creator: maybe_string
        >> hide_extension: boolean
        >> st_dev: be_i32
        >> st_ino: be_i32
        >> st_nlink: be_u32
        >> st_rdev: be_i32
        >> ctime_sec: be_i64
        >> ctime_nsec: be_i64
        >> create_time_sec: be_i64
        >> create_time_nsec: be_i64
        >> st_blocks: be_i64
        >> st_block_size: be_i32
        >> (Node {
            name,
            is_tree,
            has_missing_items,
            data_compression_type:
                unwrap_compression_type(data_is_compressed, data_compression_type),
            data_blob_keys: blob_keys,
            data_size,
            xattrs_compression_type:
                unwrap_compression_type(xattrs_are_compressed, xattrs_compression_type),
            xattrs_blob_key,
            xattrs_size,
            acl_compression_type:
                unwrap_compression_type(acl_is_compressed, acl_compression_type),
            acl_blob_key,
            user_id,
            group_id,
            file_mode,
            flags,
            finder_flags: ((extended_finder_flags as u64) << 32) | (finder_flags as u64),
            mod_time: as_datetime(mtime_sec, mtime_nsec),
            c_time: as_datetime(ctime_sec, ctime_nsec),
            create_time: as_datetime(create_time_sec, create_time_nsec),
            file_type,
            creator: file_creator,
            hide_extension,
            st_dev,
            st_ino,
            st_nlink,
            st_rdev,
            st_blocks,
            st_block_size,
        })
    )
);

named_args!(
    nodes(version: usize)<Vec<Node>>,
    do_parse!(
        n: be_u32
        >> v: many_m_n!(n as usize, n as usize, call!(node, version))
        >> (v)
    )
);

named!(
    tree<Tree>,
    do_parse!(
        version: call!(version_header, "TreeV".as_bytes())
            >> xattrs_compressed: cond!((12..=18).contains(&version), boolean)
            >> xattrs_compression_type: cond!(version >= 19, compression_type)
            >> acl_compressed: cond!((12..=18).contains(&version), boolean)
            >> acl_compression_type: cond!(version >= 19, compression_type)
            >> xattrs_blob_key: call!(maybe_blob_key, version)
            >> xattrs_blob_size: be_u64
            >> acl_blob_key: call!(maybe_blob_key, version)
            >> uid: be_i32
            >> gid: be_i32
            >> file_mode: be_i32
            >> mtime_sec: be_i64
            >> mtime_nsec: be_i64
            >> flags: be_u64
            >> finder_flags: be_u32
            >> extended_finder_flags: be_u32
            >> st_dev: be_i32
            >> st_ino: be_i32
            >> st_nlink: be_u32
            >> st_rdev: be_i32
            >> ctime_sec: be_i64
            >> ctime_nsec: be_i64
            >> st_blocks: be_i64
            >> st_block_size: be_u32
            >> size_on_disk: cond!((11..=16).contains(&version), be_u64)
            >> create_time_sec: be_i64
            >> create_time_nsec: be_i64
            >> missing_nodes: cond!(version >= 18, missing_nodes)
            >> nodes: call!(nodes, version)
            >> (Tree {
                version,
                xattrs_compression_type: unwrap_compression_type(
                    xattrs_compressed,
                    xattrs_compression_type
                ),
                acl_compression_type: unwrap_compression_type(acl_compressed, acl_compression_type),
                xattrs_blob_key,
                xattrs_blob_size,
                acl_blob_key,
                user_id: uid,
                group_id: gid,
                file_mode,
                mod_time: as_datetime(mtime_sec, mtime_nsec),
                flags,
                finder_flags: ((extended_finder_flags as u64) << 32) | (finder_flags as u64),
                st_dev,
                st_ino,
                st_nlink,
                st_rdev,
                c_time: as_datetime(ctime_sec, ctime_nsec),
                st_blocks,
                st_block_size: st_block_size as usize,
                size_on_disk: size_on_disk.unwrap_or(0),
                creation_time: as_datetime(create_time_sec, create_time_nsec),
                missing_nodes: missing_nodes.unwrap_or_else(Vec::new),
                nodes,
            })
    )
);

pub fn parse(data: &[u8]) -> Result<Tree, RepoError> {
    tree(data)
        .map_err(|_| RepoError::MalformedData)
        .map(|(_, t)| t)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CompressionType;

    const VALID_ROOT_TREE_V18: &[u8] = include_bytes!("tree_v18_parent.blob");
    const VALID_CHILD_TREE_V18: &[u8] = include_bytes!("tree_v18_child.blob");

    #[test]
    fn parse_v18_root_tree() {
        match tree(VALID_ROOT_TREE_V18) {
            Ok((_, t)) => {
                assert_eq!(t.version, 18);
                assert_eq!(t.xattrs_compression_type, CompressionType::None);
                assert_eq!(t.acl_compression_type, CompressionType::None);

                println!("Tree: {:?}", t)
            }
            Err(e) => {
                assert!(false, "Parse failed with {:?}", e);
            }
        }
    }

    #[test]
    fn parse_v18_child_tree() {
        match tree(VALID_CHILD_TREE_V18) {
            Ok((_, t)) => {
                assert_eq!(t.version, 18);
                assert_eq!(t.xattrs_compression_type, CompressionType::None);
                assert_eq!(t.acl_compression_type, CompressionType::None);

                println!("Tree: {:?}", t)
            }
            Err(e) => {
                assert!(false, "Parse failed with {:?}", e);
            }
        }
    }

    #[test]
    fn parse_v18_node() {
        let input = &VALID_ROOT_TREE_V18[0xA2..0x193];
        match node(input, 18) {
            Ok((remainder, n)) => {
                assert_eq!(remainder.len(), 0, "All input should be consumed");
                assert!(n.is_tree);
                assert_eq!(n.name, "2004");
                assert_eq!(n.data_size, 6717642793);
                assert_eq!(n.data_compression_type, CompressionType::GZip);

                //println!("Node: {:?}", n);
            }
            Err(e) => {
                assert!(false, "Parse failed with {:?}", e);
            }
        }
    }

    #[test]
    fn parse_v18_blob_key() {
        let input = &VALID_ROOT_TREE_V18[0xB8..0xF8];
        match blob_key(input, 18) {
            Ok((remainder, k)) => {
                assert_eq!(remainder.len(), 0, "Input must be fully consumed");
                assert!(k.stretch_key);
            }
            Err(e) => {
                assert!(false, "Parse failed with {:?}", e);
            }
        }
    }
}
