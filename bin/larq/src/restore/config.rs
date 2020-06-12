use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StorageClass {
    Standard,
    Glacier,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct Config {
    pub class: StorageClass,
    pub access_key_id: String,
    pub secret_key: String,
    pub region: String,
    pub bucket_name: String,
}

#[derive(Debug)]
pub enum ConfigErr {
    File(io::Error),
    Format(String),
}

pub fn load(filename: &Path) -> Result<Config, ConfigErr> {
    debug!("Loading config from {:?}", filename);
    let mut f = File::open(filename).map_err(ConfigErr::File)?;
    let mut content = String::new();

    f.read_to_string(&mut content).map_err(ConfigErr::File)?;

    parse(&content)
}

fn parse(text: &str) -> Result<Config, ConfigErr> {
    debug!("Parsing config content");

    toml::from_str::<Config>(text).map_err(|e| {
        error!("Parsing error: {}", e.description());
        ConfigErr::Format(e.description().to_string())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_storage_class_standard() {
        assert_eq!(
            StorageClass::Standard,
            toml::from_str::<StorageClass>("\"standard\"").unwrap()
        );
    }

    #[test]
    fn parse_storage_class_glacier() {
        assert_eq!(
            StorageClass::Glacier,
            toml::from_str::<StorageClass>("\"glacier\"").unwrap()
        );
    }

    #[test]
    fn parse_config() {
        let text = " \
                    region = \"ap-southeast-2\"\n \
                    access_key_id = \"ACCESS_KEY_ID\"\n \
                    secret_key = \"secret_key\"\n \
                    class = \"glacier\"\n \
                    bucket_name = \"some-bucket\"\n";

        let cfg = toml::from_str::<Config>(text).unwrap();
        let expected = Config {
            region: "ap-southeast-2".to_string(),
            access_key_id: "ACCESS_KEY_ID".to_string(),
            secret_key: "secret_key".to_string(),
            class: StorageClass::Glacier,
            bucket_name: "some-bucket".to_string(),
        };

        assert_eq!(expected, cfg)
    }
}
