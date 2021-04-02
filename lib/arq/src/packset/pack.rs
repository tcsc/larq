use nom::{
    number::streaming::{be_u64, be_u8},
};

use crate::{constructs::maybe_string, RepoError};

#[derive(Debug)]
pub struct PackedObject {
    pub mime_type: Option<String>,
    pub name: Option<String>,
    pub content: Vec<u8>,
}

fn packed_object(i: &[u8]) -> nom::IResult<&[u8], PackedObject> {
    let (i, mime_type) = maybe_string(i)?;
    let (i, name) = maybe_string(i)?;
    let (i, len) = be_u64(i).map(|(i, x)| (i, x as usize))?;
    let (i, content) = nom::multi::many_m_n(len, len, be_u8)(i)?;
    let result = PackedObject {
        mime_type,
        name,
        content
    };
    Ok((i, result))
}

pub fn parse_object(data: &[u8]) -> Result<PackedObject, RepoError> {
    log::info!("Parsing {} byte packed object", data.len());
    packed_object(data).map(|(_, obj)| obj).map_err(|e| {
        log::error!("Failed parsing pack object: {:?}", e);
        RepoError::MalformedData
    })
}

#[cfg(test)]
mod test {
    const VALID_PACK_BLOB: &[u8] = include_bytes!("pack.blob");

    #[test]
    fn parse_packed_object() {
        let (_, obj) = super::packed_object(&VALID_PACK_BLOB[16..]).expect("Parse to succeed");
        assert_eq!(obj.mime_type, None);
        assert_eq!(obj.name, None);
        assert_eq!(obj.content.len(), 192);
        assert_eq!(obj.content[0], 0x4F);
        assert_eq!(obj.content[191], 0x2B);
    }
}
