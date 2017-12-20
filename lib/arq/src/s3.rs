use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use hyper::Client;
use rusoto_core::{AwsCredentials, CredentialsError, ProvideAwsCredentials, Region, default_tls_client};
use rusoto_s3::{
    S3Client, S3,
    GetObjectRequest, GetObjectError,
    ListObjectsV2Request, ListObjectsV2Error
};

use super::{Include, StorageObject, TransportError, Key};

struct Credentials {
    access_key_id: String,
    secret_key: String,
}

impl ProvideAwsCredentials for Credentials {
    fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        let expiry = SystemTime::now() + Duration::from_secs(60 * 60);
        let creds = AwsCredentials::new(self.access_key_id.clone(),
                                        self.secret_key.clone(),
                                        None,
                                        DateTime::<Utc>::from(expiry));
        Ok(creds)
    }
}

pub struct Transport {
    bucket: String,
    s3: S3Client<Credentials, Client>,
}

impl Transport {
    pub fn new(bucket: &str, key_id: &str, secret: &str, region: Region) -> Transport {
        let creds = Credentials {
            access_key_id: key_id.to_string(),
            secret_key: secret.to_string(),
        };

        let dispatcher = default_tls_client().unwrap();
        let client = S3Client::new(dispatcher, creds, region);

        Transport {
            bucket: bucket.to_string(),
            s3: client,
        }
    }
}

impl super::Store for Transport {
    fn list_contents(&self, prefix: &str, flags: Include) ->
            Result<Vec<StorageObject>, TransportError> {
        let delimiter = '/';
        let mut continuation_token = None;
        let mut result = Vec::new();

        fn translate_err(e: ListObjectsV2Error) -> TransportError {
            match e {
                ListObjectsV2Error::NoSuchBucket(_) => TransportError::NoSuchObject,
                ListObjectsV2Error::HttpDispatch(_) => TransportError::NetworkError,
                ListObjectsV2Error::Credentials(_) => TransportError::AccessDenied,
                _ => {
                    error!("Unexpected error: {:?}", e);
                    TransportError::UnknownError
                }
            }
        }

        loop {
            let req = ListObjectsV2Request {
                bucket: self.bucket.clone(),
                continuation_token,
                delimiter: Some(delimiter.to_string()),
                prefix: Some(String::from(prefix)),
                ..ListObjectsV2Request::default()
            };

            let response = self.s3.list_objects_v2(&req)
                .map_err(translate_err)?;

            if flags.contains(Include::DIRS) {
                if let Some(prefixes) = response.common_prefixes {
                    result.extend(
                        prefixes.into_iter()
                            .map(|pfx| StorageObject {
                                key: Key::from(pfx.prefix.unwrap_or(String::from(""))),
                                size: 0
                            })
                    );
                }
            }

            if flags.contains(Include::FILES) {
                if let Some(objects) = response.contents {
                    result.extend(
                        objects.into_iter()
                            .map(|obj| super::StorageObject {
                                key: Key::from(obj.key.unwrap_or(String::from(""))),
                                size: obj.size.unwrap_or(0)
                            })
                    );
                };
            }

            if response.is_truncated.unwrap_or(false) {
                continuation_token = response.continuation_token;
            } else {
                break
            }
        }

        Ok(result)
    }

    fn get(&self, key: Key) -> Result<Vec<u8>, TransportError> {
        fn to_transport_error(e: GetObjectError) -> TransportError {
            match e {
                GetObjectError::NoSuchKey(_) => TransportError::NoSuchObject,
                GetObjectError::HttpDispatch(_) => TransportError::NetworkError,
                GetObjectError::Credentials(_) => TransportError::AccessDenied,
                _ => {
                    error!("Unexpected error: {:?}", e);
                    TransportError::UnknownError
                }
            }
        }

        let req = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: key.to_string(),
            ..GetObjectRequest::default()
        };

        let response = self.s3.get_object(&req).map_err(to_transport_error)?;
        let mut content = Vec::new();
        if let Some(mut body) = response.body {
            body.read_to_end(&mut content);
        }

        Ok(content)
    }
}


#[cfg(test)]
mod test {

}