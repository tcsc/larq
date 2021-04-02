mod cache;

use arq_storage::{Error as StorageError, Include, Key, ObjectInfo, Result as StorageResult};
use log::{debug, error};

use rusoto_core::{
    credential::StaticProvider,
    request::{HttpClient, TlsError},
    Region, RusotoError,
};
use rusoto_s3::{
    CommonPrefix, GetObjectError, GetObjectRequest, ListObjectsV2Error, ListObjectsV2Request,
    Object as S3Object, S3Client, S3,
};

use cache::Cache;

use trait_async::trait_async;

pub struct Store {
    bucket: String,
    s3: S3Client,
    cache: Cache,
}

impl Store {
    pub fn new(
        bucket: &str,
        key_id: &str,
        secret: &str,
        region: Region,
        cache_dir: Option<std::path::PathBuf>,
    ) -> Result<Store, TlsError> {
        let creds = StaticProvider::new(key_id.to_string(), secret.to_string(), None, None);
        let dispatcher = HttpClient::new()?;
        let client = S3Client::new_with(dispatcher, creds, region);

        let t = Store {
            bucket: bucket.to_string(),
            s3: client,
            cache: Cache::new(cache_dir),
        };

        Ok(t)
    }
}

fn translate_list_objects_err(err: RusotoError<ListObjectsV2Error>) -> StorageError {
    use ListObjectsV2Error::NoSuchBucket;
    use RusotoError::Service;

    match err {
        Service(NoSuchBucket(_)) => StorageError::NoSuchObject,
        _ => {
            error!("Unexpected error: {:?}", err);
            StorageError::UnknownError
        }
    }
}

fn translate_get_object_err(err: RusotoError<GetObjectError>) -> StorageError {
    use GetObjectError::NoSuchKey;
    use RusotoError::Service;

    match err {
        Service(NoSuchKey(_)) => StorageError::NoSuchObject,
        _ => {
            error!("Unexpected error: {:?}", err);
            StorageError::UnknownError
        }
    }
}

async fn read_all(mut s: rusoto_core::ByteStream) -> Result<Vec<u8>, std::io::Error> {
    use futures::stream::TryStreamExt;

    let mut result: Vec<u8> = Vec::new();

    while let Some(bs) = s.try_next().await? {
        result.extend_from_slice(bs.as_ref());
    }

    Ok(result)
}

#[trait_async]
impl arq_storage::Store for Store {
    // fn clone(&self) -> Box<dyn super::Store> {
    //     Box::new(Transport {
    //         bucket: self.bucket.clone(),
    //         s3: self.s3.clone()
    //     })
    // }

    async fn list_contents(&self, prefix: &str, flags: Include) -> StorageResult<Vec<ObjectInfo>> {
        debug!("Fetching listing for {}", prefix);
        fn object_from_pfx(pfx: CommonPrefix) -> ObjectInfo {
            ObjectInfo {
                key: Key::from(pfx.prefix.unwrap_or_default()),
                size: 0,
            }
        }

        fn object_from_content(obj: S3Object) -> ObjectInfo {
            ObjectInfo {
                key: Key::from(obj.key.unwrap_or_default()),
                size: obj.size.unwrap_or(0),
            }
        }

        let s3_client = self.s3.clone();
        let bucket = self.bucket.clone();
        let delimiter = '/'.to_string();
        let search_prefix = prefix.to_string();

        let mut result = vec![];
        let mut continuation_token = None;
        loop {
            let req = ListObjectsV2Request {
                bucket: bucket.clone(),
                continuation_token,
                delimiter: Some(delimiter.clone()),
                prefix: Some(search_prefix.clone()),
                ..ListObjectsV2Request::default()
            };

            let response = s3_client
                .list_objects_v2(req)
                .await
                .map_err(translate_list_objects_err)?;

            if flags.contains(Include::DIRS) {
                if let Some(prefixes) = response.common_prefixes {
                    result.extend(prefixes.into_iter().map(object_from_pfx));
                }
            }

            if flags.contains(Include::FILES) {
                if let Some(objects) = response.contents {
                    result.extend(objects.into_iter().map(object_from_content));
                }
            }

            if !response.is_truncated.unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token;
        }

        return Ok(result);
    }

    async fn get(&self, key: Key) -> StorageResult<Vec<u8>> {
        if let Some(buf) = self.cache.read(&key) {
            return Ok(buf);
        }

        let req = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: key.to_string(),
            ..GetObjectRequest::default()
        };

        let response = self
            .s3
            .get_object(req)
            .await
            .map_err(translate_get_object_err)?;

        let content = match response.body {
            None => Vec::new(),
            Some(body) => read_all(body)
                .await
                .map_err(|_| StorageError::NetworkError)?,
        };

        self.cache.write(&key, &content);

        Ok(content)
    }
}

#[cfg(test)]
mod test {}
