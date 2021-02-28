use std::{collections::HashMap, convert::TryInto};

use futures::future::{self, TryFutureExt};

use log::info;

use nom::{
    do_parse, many_m_n, named,
    number::streaming::{be_u32, be_u64, be_u8},
    tag, take,
};

use uuid::Uuid;

use crate::{
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

named!(
    packed_index_item<PackedIndexItem>,
    do_parse!(
        offset: be_u64
            >> length: be_u64
            >> sha: many_m_n!(20, 20, be_u8)
            >> take!(4)
            >> (PackedIndexItem {
                sha: sha.try_into().unwrap(),
                offset: offset,
                length: length,
            })
    )
);

named!(
    packed_index<PackedIndex>,
    do_parse!(
        tag!(&[0xff, 0x74, 0x4f, 0x63])
            >> version: be_u32
            >> counts: many_m_n!(256, 256, be_u32)
            >> entries:
                many_m_n!(
                    counts[0xFF] as usize,
                    counts[0xFF] as usize,
                    packed_index_item
                )
            >> (PackedIndex {
                version: version,
                counts: counts.try_into().unwrap(),
                entries: entries,
            })
    )
);

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
    let index_data = future::try_join_all(fetch_tasks)
        .await
        .map_err(RepoError::Storage)?;

    info!("Unpacking {} index files", index_data.len());

    // unpack the indices so we can use them to find files.
    let mut index_map = HashMap::new();
    for (object_key, blob) in index_data.into_iter() {
        let (pack_id, index_data) = parse(&object_key, blob)?;
        for e in index_data.entries {
            let loc = PackedItem {
                pack_id,
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
        .map(|v| v.try_into().expect("pack ID should be a SHA1")).ok()
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
