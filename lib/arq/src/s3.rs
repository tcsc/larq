use super::{Include, Key, StorageError, StorageObject};
use bytes::Bytes;
use futures::{
    future::{self, loop_fn, Either, Loop},
    Future, Stream,
};
use log::{debug, error};
use rusoto_core::{
    request::{HttpClient, TlsError},
    Region, RusotoError,
};
use rusoto_credential::StaticProvider;
use rusoto_s3::{
    CommonPrefix, GetObjectError, GetObjectRequest, ListObjectsV2Error, ListObjectsV2Request,
    Object as S3Object, S3Client, S3,
};
use std::io;

pub struct Transport {
    bucket: String,
    s3: S3Client,
}

impl Transport {
    pub fn new(
        bucket: &str,
        key_id: &str,
        secret: &str,
        region: Region,
    ) -> Result<Transport, TlsError> {
        let creds = StaticProvider::new(key_id.to_string(), secret.to_string(), None, None);
        let dispatcher = HttpClient::new()?;
        let client = S3Client::new_with(dispatcher, creds, region);

        let t = Transport {
            bucket: bucket.to_string(),
            s3: client,
        };

        Ok(t)
    }
}

impl super::Store for Transport {
    // fn clone(&self) -> Box<dyn super::Store> {
    //     Box::new(Transport {
    //         bucket: self.bucket.clone(),
    //         s3: self.s3.clone()
    //     })
    // }

    fn list_contents(
        &self,
        prefix: &str,
        flags: Include,
    ) -> super::StorageFuture<Vec<StorageObject>> {
        fn object_from_pfx(pfx: CommonPrefix) -> StorageObject {
            StorageObject {
                key: Key::from(pfx.prefix.unwrap_or_default()),
                size: 0,
            }
        }

        fn object_from_content(obj: S3Object) -> StorageObject {
            StorageObject {
                key: Key::from(obj.key.unwrap_or_default()),
                size: obj.size.unwrap_or(0),
            }
        }

        let s3_client = self.s3.clone();
        let bucket = self.bucket.clone();
        let delimiter = '/'.to_string();
        let search_prefix = prefix.to_string();

        let f = loop_fn(
            (Vec::new(), None),
            move |(mut result, continuation_token)| {
                let req = ListObjectsV2Request {
                    bucket: bucket.clone(),
                    continuation_token,
                    delimiter: Some(delimiter.clone()),
                    prefix: Some(search_prefix.clone()),
                    ..ListObjectsV2Request::default()
                };

                s3_client
                    .list_objects_v2(req)
                    .map_err(|e| match e {
                        RusotoError::Service(ListObjectsV2Error::NoSuchBucket(_)) => {
                            StorageError::NoSuchObject
                        }
                        _ => {
                            error!("Unexpected error: {:?}", e);
                            StorageError::UnknownError
                        }
                    })
                    .and_then(move |response| {
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

                        if response.is_truncated.unwrap_or(false) {
                            Ok(Loop::Continue((result, response.continuation_token)))
                        } else {
                            Ok(Loop::Break(result))
                        }
                    })
            },
        );

        Box::new(f)
    }

    fn get(&self, key: Key) -> super::StorageFuture<Vec<u8>> {
        debug!("Fetching object for key {:?}", key);
        let req = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: key.to_string(),
            ..GetObjectRequest::default()
        };

        let f = self
            .s3
            .get_object(req)
            .map_err(|e| match e {
                RusotoError::Service(GetObjectError::NoSuchKey(_)) => StorageError::NoSuchObject,
                _ => {
                    error!("Unexpected error: {:?}", e);
                    StorageError::UnknownError
                }
            })
            .and_then(|response| match response.body {
                Some(body) => {
                    debug!("Reading object body...");
                    fn append_body(
                        mut acc: Vec<u8>,
                        bytes: Bytes,
                    ) -> impl Future<Item = Vec<u8>, Error = io::Error> {
                        acc.extend(bytes);
                        future::ok(acc)
                    };

                    let collect_body = body
                        .fold(Vec::new(), append_body)
                        .map_err(|_| StorageError::NetworkError);

                    Either::A(collect_body)
                }
                None => Either::B(future::ok(Vec::new())),
            });

        Box::new(f)
    }
}

#[cfg(test)]
mod test {}
