use std::{collections::HashMap, convert::TryInto};

use futures::future::TryFutureExt;

use log::info;

use nom::{
    combinator::{map_res},
    bytes::streaming::{tag, take},
    multi::many_m_n,
    number::streaming::{be_u32, be_u64},
    IResult
};

use crate::{
    constructs::binary_sha1,
    storage::{Include, Key, Store},
    RepoError, SHA1,
};

#[derive(Debug, Clone)]
struct PackedIndex {
    version: u32,
    counts: [u32; 256],
    entries: Vec<PackedIndexItem>,
}

#[derive(Debug, Clone)]
struct PackedIndexItem {
    sha: SHA1,
    offset: u64,
    length: u64,
}

pub struct PackedItem {
    pub pack_id: SHA1,
    pub offset: u64,
    pub length: u64,
}

pub type PackIndex = HashMap<SHA1, PackedItem>;

fn packed_index_item(i: &[u8]) -> IResult<&[u8], PackedIndexItem> {
    let (i, offset) = be_u64(i)?;
    let (i, length) = be_u64(i)?;
    let (i, sha) = binary_sha1(i)?;
    let (i, _) = take(4usize)(i)?;
    let item = PackedIndexItem { sha, offset, length };
    Ok((i, item))
}

fn packed_index(i: &[u8]) -> IResult<&[u8], PackedIndex> {
    let (i, _) = tag(&[0xff, 0x74, 0x4f, 0x63])(i)?;
    let (i, version) = be_u32(i)?;
    let (i, counts) = map_res(many_m_n(256, 256, be_u32), TryInto::<[u32; 256]>::try_into)(i)?;
    let count = counts[0xFF] as usize;
    let (i, entries) = many_m_n(count, count, packed_index_item)(i)?;
    let idx = PackedIndex { version, counts, entries };
    Ok((i, idx))
}

fn parse_blob(data: &[u8]) -> Option<PackedIndex> {
    packed_index(data).map(|(_, i)| i).ok()
}

pub async fn load(key: &Key, store: &dyn Store) -> Result<PackIndex, RepoError> {
    // list the contents of the
    let objects = store
        .list_contents(key.as_str(), Include::FILES)
        .await
        .map_err(RepoError::Storage)?;

    let fetch_tasks = objects
        .into_iter()
        .filter(|o| o.key.ends_with(".index"))
        .map(|o| store.get(o.key.clone()).map_ok(|data| (o.key, data)));

    // check if any of the index fetches failed
    let index_data = throttled::try_join_all(5, fetch_tasks)
        .await
        .map_err(RepoError::Storage)?;

    info!("Unpacking {} index files", index_data.len());

    // unpack the indices so we can use them to find files.
    let mut index_map = HashMap::new();
    for (object_key, blob) in index_data.into_iter() {
        let (pack_id, index_data) = parse(&object_key, blob)?;
        for e in index_data.entries {
            let loc = PackedItem {
                pack_id: pack_id.clone(),
                offset: e.offset,
                length: e.length,
            };
            index_map.insert(e.sha, loc);
        }
    }

    info!("Indexed {} objects", index_map.len());

    Ok(index_map)
}

fn parse(key: &Key, blob: Vec<u8>) -> Result<(SHA1, PackedIndex), RepoError> {
    extract_pack_id(key)
        .and_then(|k| parse_blob(&blob).map(|i| (k, i)))
        .ok_or(RepoError::MalformedData)
}

fn extract_pack_id(key: &Key) -> Option<SHA1> {
    let s = key.as_str();
    let start = s.rfind('/')?;
    let end = s.rfind('.')?;
    let substr = &s[start + 1..end];
    hex::decode(substr)
        .map(|v| v.try_into().expect("pack ID should be a SHA1"))
        .ok()
}

#[cfg(test)]
mod test {
    const VALID_INDEX_BLOB: &[u8] = include_bytes!("index.blob");

    #[test]
    fn parse() {
        let r = super::parse_blob(VALID_INDEX_BLOB).unwrap();
        for e in r.entries.iter() {
            println!("{:?}", e);
        }
        assert_eq!(r.entries.len(), 5);
        assert!(r.counts[0..28].iter().all(|x| *x == 0));
        assert!(r.counts[28..66].iter().all(|x| *x == 1));
        assert!(r.counts[66..107].iter().all(|x| *x == 2));
        assert!(r.counts[107..143].iter().all(|x| *x == 3));
        assert!(r.counts[143..217].iter().all(|x| *x == 4));
        assert!(r.counts[217..255].iter().all(|x| *x == 5));
    }
}
