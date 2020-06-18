use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Computer {
    #[serde(skip)]
    pub id: String,

    #[serde(skip)]
    pub salt: Vec<u8>,

    #[serde(rename = "userName")]
    pub user: String,

    #[serde(rename = "computerName")]
    pub computer: String,
}
