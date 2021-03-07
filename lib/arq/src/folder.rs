use std::{convert::TryInto, path::PathBuf, sync::Arc};

use crate::{
    crypto::ObjectDecrypter,
    format_uuid,
    packset::Packset,
    storage::{self, Store},
    tree::TreeIndex,
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
    store: Arc<dyn Store>,
    decrypter: Arc<dyn ObjectDecrypter>,
    computer_id: String,
}

pub type CommitId = SHA1;

impl Folder {
    pub fn new(
        computer_id: &str,
        info: FolderInfo,
        store: &Arc<dyn Store>,
        decrypter: &Arc<dyn ObjectDecrypter>,
    ) -> Folder {
        Folder {
            info,
            store: store.clone(),
            decrypter: decrypter.clone(),
            computer_id: computer_id.to_owned(),
        }
    }

    pub async fn get_latest_commit(&self) -> Result<CommitId, RepoError> {
        let key = storage::Key::from(format!(
            "{}/bucketdata/{}/refs/heads/master",
            self.computer_id,
            format_uuid(&self.info.id)
        ));
        let content = self.store.get(key).await.map_err(RepoError::Storage)?;

        String::from_utf8(content)
            .ok()
            .and_then(|s| hex::decode(&s[..s.len() - 1]).ok())
            .and_then(|v| v.try_into().ok())
            .ok_or(RepoError::MalformedData)
    }

    //    pub async fn list_commits(&self) -> Result<CommitId, RepoError> {}

    pub async fn load_tree_index(&mut self) -> Result<TreeIndex, RepoError> {
        info!("Fetching tree pack index");
        let key = storage::Key::from(format!(
            "{}/packsets/{}-trees/",
            self.computer_id,
            format_uuid(&self.info.id)
        ));

        // load the commit packset (at least the index)
        Packset::new(key, &self.store)
            .await
            .map(|ps| TreeIndex::new(ps, &self.decrypter))
    }
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
