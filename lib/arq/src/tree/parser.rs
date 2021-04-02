use std::convert::TryFrom;

use crate::{
    constructs::*,
    tree::{BlobKey, Node, StorageType, Tree},
    CompressionType,
    RepoError,
};

use nom::{
    combinator::{cond, map, map_res},
    multi::many_m_n,
    number::streaming::{be_i32, be_i64, be_u32, be_u64},
    IResult
};

use chrono::prelude::*;

fn storage_type(i: &[u8]) -> IResult<&[u8], StorageType> {
    map_res(be_u32, TryFrom::<u32>::try_from)(i)
}

// NULL Blobkeys are written as a string of zero values, rather than having a
// "present" flag at the start. I assume that this is due to a bug in Arq from
// a decade ago that we're stuck with.
fn maybe_blob_key(tree_version: usize) -> 
    impl FnMut(&[u8]) -> IResult<&[u8], Option<BlobKey>> 
{
    move |i: &[u8]| {
        let (i, maybe_sha) = maybe_sha_string(i)?;
        let (i, expand) = cond(tree_version >= 14, boolean)(i)?;
        let (i, storage_type) = cond(tree_version >= 17, storage_type)(i)?;
        let (i, _archive_id) = cond(tree_version >= 17, maybe_string)(i)?;
        let (i, size) = cond(tree_version >= 17, be_u64)(i)?;
        let (i, date) = cond(tree_version >= 17, maybe_date_time)(i)?;
        let result = maybe_sha.map(|sha| {
            BlobKey {
                sha,
                stretch_key: expand.unwrap_or(false),
                storage_type: storage_type.unwrap_or(StorageType::S3),
                size,
                upload_date: date.unwrap_or(None),
            }
        });
        Ok((i, result))
    }
}

fn blob_key<'a>(tree_version: usize) -> impl FnMut(&'a [u8]) -> IResult<&'a[u8], BlobKey> {
    map_res(maybe_blob_key(tree_version), |b: Option<BlobKey>| {
        b.ok_or("Blob key may not be null")
    })
}

fn as_datetime(s: i64, ns: i64) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(s, ns as u32), Utc)
}

fn missing_nodes(i: &[u8]) -> IResult<&[u8], Vec<String>> {
    let (i, n) = map(be_u32, |x| x as usize)(i)?;
    many_m_n(n, n, non_null_string)(i)
}

fn blob_keys(tree_version: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<BlobKey>> {
    move |i: &[u8]| {
        let (i, n) = map(be_u32, |x| x as usize)(i)?;
        many_m_n(n, n, blob_key(tree_version))(i)
    }
}

fn compression_flag_or_type(tree_version: usize) -> 
    impl FnMut(&[u8]) -> IResult<&[u8], CompressionType> 
{
    move |i: &[u8]| {
        if tree_version >= 19 {
            compression_type(i)
        } else if (12..=18).contains(&tree_version) {
            map(boolean, |b| {
                if b { 
                    CompressionType::GZip
                } else {
                    CompressionType::None
                }
            })(i)
        } else {
            Ok((i, CompressionType::None))
        }
    }
}

fn node(tree_version: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Node> {
    let mut ctype = compression_flag_or_type(tree_version);

    move |i: &[u8]| {
        let (i, name) = non_null_string(i)?;
        let (i, is_tree) = boolean(i)?;
        let (i, has_missing_items) = cond(tree_version >= 18, boolean)(i)?;
        let (i, data_compression_type) = ctype(i)?;
        let (i, xattrs_compression_type) = ctype(i)?;
        let (i, acl_compression_type) = ctype(i)?;
        let (i, blob_keys) = blob_keys(tree_version)(i)?;
        let (i, data_size) = be_u64(i)?;
        // NB: The docs say <= v18, but the arq restore implemetation says a strictly
        // less than, and the actual v18 blobs agree
        let (i, _thumbnail_sha) = cond(tree_version < 18, maybe_sha_string)(i)?;
        let (i, _stretch_thumbnail_key) = cond((14..=17).contains(&tree_version), boolean)(i)?;
        let (i, _preview_sha) = cond(tree_version < 18, maybe_sha_string)(i)?;
        let (i, _stretch_preview_key) = cond((14..=17).contains(&tree_version), boolean)(i)?;
        let (i, xattrs_blob_key) = maybe_blob_key(tree_version)(i)?;
        let (i, xattrs_size) = be_u64(i)?;
        let (i, acl_blob_key) = maybe_blob_key(tree_version)(i)?;
        let (i, user_id) = be_i32(i)?;
        let (i, group_id) = be_i32(i)?;
        let (i, file_mode) = be_i32(i)?;
        let (i, mtime_sec) = be_i64(i)?;
        let (i, mtime_nsec) = be_i64(i)?;
        let (i, flags) = be_u64(i)?;
        let (i, finder_flags) = be_u32(i)?;
        let (i, extended_finder_flags) = be_u32(i)?;
        let (i, file_type) = maybe_string(i)?;
        let (i, file_creator) = maybe_string(i)?;
        let (i, hide_extension) = boolean(i)?;
        let (i, st_dev) = be_i32(i)?;
        let (i, st_ino) = be_i32(i)?;
        let (i, st_nlink) = be_u32(i)?;
        let (i, st_rdev) = be_i32(i)?;
        let (i, ctime_sec) = be_i64(i)?;
        let (i, ctime_nsec) = be_i64(i)?;
        let (i, create_time_sec) = be_i64(i)?;
        let (i, create_time_nsec) = be_i64(i)?;
        let (i, st_blocks) = be_i64(i)?;
        let (i, st_block_size) = be_i32(i)?;
        let n = Node {
            name,
            is_tree,
            has_missing_items,
            data_compression_type,
            data_blob_keys: blob_keys,
            data_size,
            xattrs_compression_type,
            xattrs_blob_key,
            xattrs_size,
            acl_compression_type,
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
        };
        Ok((i, n)) 
    }
}

fn nodes(tree_version: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<Node>> {
    move |i: &[u8]| {
        let (i, n) = map(be_u32, |x| x as usize)(i)?;
        many_m_n(n, n, node(tree_version))(i)
    }
}

const TREE_HEADER_PREFIX : &[u8] = "TreeV".as_bytes();

fn tree(i: &[u8]) -> IResult<&[u8], Tree> {
    let (i, version) = version_header(TREE_HEADER_PREFIX)(i)?;

    let mut ctype = compression_flag_or_type(version);
    let mut bkey = maybe_blob_key(version);

    let (i, xattrs_compression_type) = ctype(i)?;
    let (i, acl_compression_type) = ctype(i)?;
    let (i, xattrs_blob_key) = bkey(i)?;
    let (i, xattrs_blob_size) = be_u64(i)?;
    let (i, acl_blob_key) = bkey(i)?;
    let (i, uid) = be_i32(i)?;
    let (i, gid) = be_i32(i)?;
    let (i, file_mode) = be_i32(i)?;
    let (i, mtime_sec) = be_i64(i)?;
    let (i, mtime_nsec) = be_i64(i)?;
    let (i, flags) = be_u64(i)?;
    let (i, finder_flags) = be_u32(i)?;
    let (i, extended_finder_flags) = be_u32(i)?;
    let (i, st_dev) = be_i32(i)?;
    let (i, st_ino) = be_i32(i)?;
    let (i, st_nlink) = be_u32(i)?;
    let (i, st_rdev) = be_i32(i)?;
    let (i, ctime_sec) = be_i64(i)?;
    let (i, ctime_nsec) = be_i64(i)?;
    let (i, st_blocks) = be_i64(i)?;
    let (i, st_block_size) = be_u32(i)?;
    let (i, size_on_disk) = cond((11..=16).contains(&version), be_u64)(i)?;
    let (i, create_time_sec) = be_i64(i)?;
    let (i, create_time_nsec) = be_i64(i)?;
    let (i, missing_nodes) = cond(version >= 18, missing_nodes)(i)?;
    let (i, nodes) = nodes(version)(i)?;

    let t = Tree {
        version,
        xattrs_compression_type,
        acl_compression_type,
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
    };

    Ok((i, t))
}

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
        match node(18)(input) {
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
        match blob_key(18)(input) {
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
