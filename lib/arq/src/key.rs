use std::ops::Div;
use std::convert::From;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Key(String);

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
        s.push('/');
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
        assert_eq!(root/"alpha"/"beta", Key::from("root/alpha/beta"))
    }
}
