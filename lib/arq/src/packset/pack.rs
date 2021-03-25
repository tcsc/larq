use nom::{
    do_parse, many_m_n, named,
    number::streaming::{be_u64, be_u8},
};

use crate::{constructs::maybe_string, RepoError};

#[derive(Debug)]
pub struct PackedObject {
    pub mime_type: Option<String>,
    pub name: Option<String>,
    pub content: Vec<u8>,
}

named!(
    packed_object<PackedObject>,
    do_parse!(
        mime_type: maybe_string
            >> name: maybe_string
            >> len: be_u64
            >> content: many_m_n!(len as usize, len as usize, be_u8)
            >> (PackedObject {
                mime_type,
                name,
                content
            })
    )
);

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
