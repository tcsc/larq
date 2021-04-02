use std::{
    convert::TryInto,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    commit::Commit,
    crypto::ObjectDecrypter,
    format_uuid,
    packset::Packset,
    storage::{self, Store},
    RepoError, SHA1,
};

use log::info;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct FolderInfo {
    #[serde(rename = "BucketUUID")]
    id: Uuid,

    #[serde(rename = "BucketName")]
    name: String,

    #[serde(rename = "LocalPath")]
    local_path: PathBuf,
}

pub struct Folder {
    pub info: FolderInfo,
    packset: Packset,
    decrypter: Arc<dyn ObjectDecrypter>,
    computer_id: String,
}

impl Folder {
    pub async fn new(
        computer_id: &str,
        info: FolderInfo,
        store: &Arc<dyn Store>,
        decrypter: &Arc<dyn ObjectDecrypter>,
    ) -> Result<Folder, RepoError> {
        let packset = load_packset(computer_id, &info, store).await?;
        let f = Folder {
            info,
            packset,
            decrypter: decrypter.clone(),
            computer_id: computer_id.to_owned(),
        };
        Ok(f)
    }

    pub async fn get_latest_commit(&'_ self) -> Result<Commit<'_>, RepoError> {
        let key = storage::Key::from(format!(
            "{}/bucketdata/{}/refs/heads/master",
            self.computer_id,
            format_uuid(&self.info.id)
        ));
        let content = self
            .packset
            .store()
            .get(key)
            .await
            .map_err(RepoError::Storage)?;

        let commit_sha = String::from_utf8(content)
            .ok()
            .and_then(|s| hex::decode(&s[..s.len() - 1]).ok())
            .and_then(|v| v.try_into().ok())
            .ok_or(RepoError::MalformedData)?;

        self.get_commit(commit_sha).await
    }

    pub async fn get_commit(&'_ self, commit_id: SHA1) -> Result<Commit<'_>, RepoError> {
        log::info!("Loading commit {}", commit_id);
        self.packset.load(&commit_id).await.and_then(|blob| {
            self.decrypter
                .decrypt_object(&blob.content)
                .map_err(|_e| RepoError::CryptoError)
                .and_then(|d| Commit::parse(&d, &self.packset, &self.decrypter))
        })
    }

    pub fn local_path(&self) -> &Path {
        &self.info.local_path
    }
}

async fn load_packset(
    computer_id: &str,
    info: &FolderInfo,
    store: &Arc<dyn Store>,
) -> Result<Packset, RepoError> {
    info!("Fetching tree pack index");
    let key = storage::Key::from(format!(
        "{}/packsets/{}-trees/",
        computer_id,
        format_uuid(&info.id)
    ));

    // load the metadata packset (at least the index)
    Packset::new(key, &store).await
}

#[cfg(test)]
mod test {
    use super::FolderInfo;

    #[test]
    fn test_parse() {
        use std::path::PathBuf;
        use uuid::Uuid;

        let text = r#"
        <plist version="1.0">
            <dict>
                <key>AWSRegionName</key>
                <string>us-east-1</string>
                <key>BucketUUID</key>
                <string>408E376B-ECF7-4688-902A-1E7671BC5B9A</string>
                <key>BucketName</key>
                <string>company</string>
                <key>ComputerUUID</key>
                <string>600150F6-70BB-47C6-A538-6F3A2258D524</string>
                <key>LocalPath</key>
                <string>/Users/stefan/src/company</string>
                <key>LocalMountPoint</key>
                <string>/</string>
                <key>StorageType</key>
                <integer>1</integer>
                <key>VaultName</key>
                <string>arq_408E376B-ECF7-4688-902A-1E7671BC5B9A</string>
                <key>VaultCreatedTime</key>
                <real>12345678.0</real>
                <key>Excludes</key>
                <dict>
                    <key>Enabled</key>
                    <false></false>
                    <key>MatchAny</key>
                    <true></true>
                    <key>Conditions</key>
                    <array></array>
                </dict>
            </dict>
        </plist>"#;

        let f: FolderInfo = plist::from_bytes(text.as_bytes()).unwrap();
        let folder_id = Uuid::parse_str("408E376B-ECF7-4688-902A-1E7671BC5B9A").unwrap();
        assert_eq!(f.id, folder_id);
        assert_eq!(f.name, "company");
        assert_eq!(f.local_path, PathBuf::from("/Users/stefan/src/company"));
    }
}
