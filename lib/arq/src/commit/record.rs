use std::convert::TryInto;

use chrono::prelude::*;

use nom::{
    call, cond, do_parse, many_m_n, map_res, named, named_args,
    number::streaming::{be_u64, be_u8},
};

use crate::{constructs::*, CompressionType, RepoError, SHA1};

#[derive(Debug)]
struct FileError {
    filename: String,
    error: String,
}

#[derive(Debug)]
pub struct CommitRecord {
    version: usize,
    author: Option<String>,
    comment: Option<String>,
    parents: Vec<ParentKey>,
    pub tree_sha: SHA1,
    pub expand_key: bool,
    pub compression_type: CompressionType,
    path: Option<String>,
    pub timestamp: DateTime<Utc>,
    file_errors: Vec<FileError>,
    missing_nodes: Option<bool>,
    is_complete: Option<bool>,
    plist: Vec<u8>,
    arq_version: Option<String>,
}

named!(
    file_error<FileError>,
    do_parse!(
        path: non_null_string
            >> error: non_null_string
            >> (FileError {
                filename: path,
                error
            })
    )
);

named!(
    file_errors<Vec<FileError>>,
    do_parse!(n: be_u64 >> errors: many_m_n!(n as usize, n as usize, file_error) >> (errors))
);

named!(
    data<Vec<u8>>,
    do_parse!(n: be_u64 >> data: many_m_n!(n as usize, n as usize, be_u8) >> (data))
);

#[derive(Debug)]
pub struct ParentKey {
    id: SHA1,
    expand_key: bool,
}

named_args!(
    parent_key(version: usize)<ParentKey>,
    do_parse!(
        sha: sha_string
        >> expand: cond!(version >= 4, boolean)
        >> ( ParentKey {
            id: sha,
            expand_key: expand.unwrap_or(false)
        })
    )
);

named!(
    commit_record<CommitRecord>,
    do_parse!(
        version: call!(version_header, "CommitV".as_bytes())
            >> author: maybe_string
            >> comment: maybe_string
            >> parent_count: be_u64
            >> parents:
                many_m_n!(
                    parent_count as usize,
                    parent_count as usize,
                    call!(parent_key, version)
                )
            >> tree_sha: map_res!(non_null_string, TryInto::<SHA1>::try_into)
            >> expand_key: cond!(version >= 4, be_u8)
            >> compressed: cond!((8..=9).contains(&version), boolean)
            >> compression_type: cond!(version >= 10, compression_type)
            >> path: maybe_string
            >> common_ancestor: cond!(version <= 7, maybe_string)
            >> common_ancestor_stretched: cond!((4..=7).contains(&version), be_u8)
            >> time_stamp: date_time
            >> file_errors: cond!(version >= 3, file_errors)
            >> missing_nodes: cond!(version >= 8, be_u8)
            >> is_complete: cond!(version >= 9, be_u8)
            >> plist: cond!(version >= 5, data)
            >> arq_version: cond!(version >= 12, non_null_string)
            >> (CommitRecord {
                version,
                author,
                comment,
                parents,
                tree_sha,
                expand_key: expand_key.map(|e| e != 0).unwrap_or(false),
                compression_type: unwrap_compression_type(compressed, compression_type),
                path,
                timestamp: time_stamp,
                file_errors: file_errors.unwrap_or_else(Vec::new),
                missing_nodes: missing_nodes.map(|e| e != 0),
                is_complete: is_complete.map(|e| e != 0),
                plist: plist.unwrap_or_else(Vec::new),
                arq_version,
            })
    )
);

pub fn parse(data: &[u8]) -> Result<CommitRecord, RepoError> {
    commit_record(data)
        .map(|(_, c)| c)
        .map_err(|_| RepoError::MalformedData)
}

#[cfg(test)]
mod test {
    const COMMIT_V9: &[u8] = include_bytes!("commit.blob");
    use crate::mocks::*;

    #[test]
    fn test_parse_v9() {
        let (_, c) = super::commit_record(COMMIT_V9).expect("Parsing commit should succeed");
        assert_eq!(c.version, 9);
    }
}
