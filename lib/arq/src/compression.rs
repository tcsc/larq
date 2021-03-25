use crate::{CompressionType, RepoError};

pub fn decompress(input: &[u8], compression_type: CompressionType) -> Result<Vec<u8>, RepoError> {
    use std::io::Write;

    match compression_type {
        CompressionType::GZip => {
            let writer = Vec::new();
            let mut decoder = flate2::write::GzDecoder::new(writer);
            decoder
                .write_all(input)
                .and_then(|_| decoder.finish())
                .map_err(|_| RepoError::MalformedData)
        }
        _ => {
            log::error!("Compression type not supported yet: {:?}", compression_type);
            Err(RepoError::MalformedData)
        }
    }
}
