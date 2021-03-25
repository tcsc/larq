use std::convert::From;
use std::ops::Div;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Default)]
pub struct Key(String);

impl Key {
    pub fn as_str(&self) -> &str {
        let &Key(ref s) = self;
        s.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn ends_with(&self, suffix: &str) -> bool {
        let &Key(ref s) = self;
        s.ends_with(suffix)
    }
}

impl ToString for Key {
    fn to_string(&self) -> String {
        let &Key(ref s) = self;
        s.clone()
    }
}

impl<'a> From<&'a str> for Key {
    fn from(s: &'a str) -> Key {
        Key::from(String::from(s))
    }
}

impl From<String> for Key {
    fn from(s: String) -> Key {
        Key(s)
    }
}

impl Div<Key> for Key {
    type Output = Key;

    fn div(self, rhs: Key) -> Key {
        let Key(ref s) = rhs;
        self / &s[..]
    }
}

impl<'a> Div<&'a str> for Key {
    type Output = Key;

    fn div(self, rhs: &'a str) -> Key {
        let Key(mut s) = self;
        s.reserve(rhs.len() + 1);
        if let Some(c) = s.chars().last() {
            if c != '/' {
                s.push('/');
            }
        }
        s += rhs;
        Key(s)
    }
}

impl<'a> Div<&'a str> for &Key {
    type Output = Key;

    fn div(self, rhs: &'a str) -> Key {
        let mut s = self.0.to_owned();
        s.reserve(rhs.len() + 1);
        if let Some(c) = s.chars().last() {
            if c != '/' {
                s.push('/');
            }
        }
        s += rhs;
        Key(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_concatenation_with_str() {
        let root = Key::from("root");
        assert_eq!(root / "alpha" / "beta", Key::from("root/alpha/beta"))
    }
}
