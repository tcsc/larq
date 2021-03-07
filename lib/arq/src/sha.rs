use std::{convert::TryFrom, fmt};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SHA1([u8; 20]);

impl SHA1 {
    pub fn as_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl TryFrom<Vec<u8>> for SHA1 {
    type Error = Vec<u8>;

    fn try_from(v: Vec<u8>) -> Result<SHA1, Vec<u8>> {
        TryFrom::try_from(v).map(SHA1)
    }
}

impl<'a> TryFrom<&'a str> for SHA1 {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<SHA1, &'static str> {
        match hex::decode(s) {
            Ok(v) => SHA1::try_from(v).map_err(|_| "Not a valid SHA1"),
            Err(_) => Err("Not a valid hex string"),
        }
    }
}

impl TryFrom<String> for SHA1 {
    type Error = &'static str;

    fn try_from(s: String) -> Result<SHA1, &'static str> {
        hex::decode(&s)
            .map_err(|_| "Not a valid hex string")
            .and_then(|v: Vec<u8>| SHA1::try_from(v).map_err(|_| "Not a valid SHA1"))
    }
}

impl fmt::Debug for SHA1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SHA1(a) = self;
        f.write_str(&hex::encode(a))
    }
}

impl fmt::Display for SHA1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SHA1(a) = self;
        f.write_str(&hex::encode(a))
    }
}
