use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Folder {
    #[serde(rename = "BucketName")]
    name: String,

    #[serde(rename = "LocalPath")]
    local_path: PathBuf,
}

#[cfg(test)]
mod test {
    use super::Folder;

    #[test]
    fn test_parse() {
        use std::path::PathBuf;

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

        let f: Folder = plist::from_bytes(text.as_bytes()).unwrap();
        assert_eq!(f.name, "company");
        assert_eq!(f.local_path, PathBuf::from("/Users/stefan/src/company"));
    }
}
