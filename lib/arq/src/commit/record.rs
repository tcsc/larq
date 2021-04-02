use std::convert::TryInto;

use chrono::prelude::*;

use nom::{
    combinator::{cond, map_res, },
    multi::{length_data, many_m_n},
    number::streaming::be_u64,
    IResult,
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

fn file_error(i: &[u8]) -> IResult<&[u8], FileError> {
    let (i, path) = non_null_string(i)?;
    let (i, error) = non_null_string(i)?;
    Ok((i, FileError { filename: path, error }))
}

fn file_errors(i: &[u8]) -> IResult<&[u8], Vec<FileError>> {
    vec_of(i, file_error)
}

#[derive(Debug)]
pub struct ParentKey {
    id: SHA1,
    expand_key: bool,
}

fn parent_key(commit_version: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], ParentKey> {
    move |i: &[u8]| {
        let (i, sha) = sha_string(i)?;
        let (i, expand) = cond(commit_version >= 4, boolean)(i)?;
        let key = ParentKey { id: sha, expand_key: expand.unwrap_or(false) };
        Ok((i, key))
    }
}

const COMMIT_PREFIX : &[u8] = "CommitV".as_bytes();

fn commit_record(i: &[u8]) -> IResult<&[u8], CommitRecord> {
    let (i, version) = version_header(COMMIT_PREFIX)(i)?;
    let (i, author) = maybe_string(i)?;
    let (i, comment) = maybe_string(i)?;
    let (i, pcount) = be_u64(i).map(|(i, x)| (i, x as usize))?;
    let (i, parents) = many_m_n(pcount, pcount, parent_key(version))(i)?;
    let (i, tree_sha) = map_res(non_null_string, TryInto::<SHA1>::try_into)(i)?;
    let (i, expand_key) = cond(version >= 4, boolean)(i)?;
    let (i, is_compressed) = cond((8..=9).contains(&version), boolean)(i)?;
    let (i, compression_type) = cond(version >= 10, compression_type)(i)?;
    let (i, path) = maybe_string(i)?;
    let (i, _common_ancestor) = cond(version <= 7, maybe_sha_string)(i)?;
    let (i, _common_ancestor_stretched) = cond((4..=7).contains(&version), boolean)(i)?;
    let (i, time_stamp) = date_time(i)?;
    let (i, file_errors) = cond(version >= 3, file_errors)(i)?;
    let (i, missing_nodes) = cond(version >= 8, boolean)(i)?;
    let (i, is_complete) = cond(version >= 9, boolean)(i)?;
    let (i, plist) =  cond(version >= 5, length_data(be_u64))(i)?;
    let (i, arq_version) = cond(version >= 12, non_null_string)(i)?;

    let compression_type = unwrap_compression_type(is_compressed, compression_type);

    let record = CommitRecord {
        version,
        author,
        comment,
        parents,
        tree_sha,
        expand_key: expand_key.unwrap_or(false),
        compression_type,
        path,
        timestamp: time_stamp,
        file_errors: file_errors.unwrap_or_else(Vec::new),
        missing_nodes,
        is_complete,
        plist: plist.map(Vec::from).unwrap_or_else(Vec::new),
        arq_version,
    };
    Ok((i, record))
}

pub fn parse(data: &[u8]) -> Result<CommitRecord, RepoError> {
    commit_record(data)
        .map(|(_, c)| c)
        .map_err(|_| RepoError::MalformedData)
}

#[cfg(test)]
mod test {
    const COMMIT_V9: &[u8] = include_bytes!("commit.blob");
    //use crate::mocks::*;

    #[test]
    fn test_parse_v9() {
        let (_, c) = super::commit_record(COMMIT_V9).expect("Parsing commit should succeed");
        assert_eq!(c.version, 9);
    }
}
