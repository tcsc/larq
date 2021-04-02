use std::{
    convert::{TryFrom, TryInto},
    str,
};

use chrono::prelude::*;

use nom::{
    bytes::streaming::{tag, take},
    combinator::{map, map_res},
    error::ParseError,
    multi::{length_data, many_m_n},
    number::streaming::{be_u32, be_u64, be_u8},
    sequence::preceded,
    IResult,
    Parser,
};

use crate::{CompressionType, SHA1};

pub fn vec_of<'a, O, E, F>(i: &'a [u8], parse: F) -> IResult<&'a [u8], Vec<O>, E>
where
    F: Parser<&'a [u8], O, E>,
    E: ParseError<&'a [u8]>
{
    let (i, count) = map(be_u64, |x| x as usize)(i)?;
    many_m_n(count, count, parse)(i)
}

pub fn boolean(i: &[u8]) -> IResult<&[u8], bool> {
    map(be_u8, |b| b != 0)(i)
}

pub fn sized_string(i: &[u8]) -> IResult<&[u8], String> {
    map_res(
        length_data(be_u64), 
        |b: &[u8]| str::from_utf8(b).map(|s| s.to_owned())
    )(i)
}

pub fn maybe_string(i: &[u8]) -> IResult<&[u8], Option<String>> {
    let (i, present) = boolean(i)?;
    if present {
        map(sized_string, Some)(i)
    } else {
        Ok((i, None))
    }
}

pub fn non_null_string(i: &[u8]) -> IResult<&[u8], String> {
    map_res(maybe_string, |s: Option<String>| {
        s.ok_or("String may not be null")
    })(i)
}

pub fn binary_sha1(i: &[u8]) -> IResult<&[u8], SHA1> {
    map_res(take(20usize), |s: &[u8]| s.try_into())(i)
}

fn unpack_timestamp(ms: u64) -> DateTime<Utc> {
    let s = (ms / 1000) as i64;
    let ns = ((ms % 1000) * 1000) as u32;
    let naive_time = NaiveDateTime::from_timestamp(s, ns);
    DateTime::<Utc>::from_utc(naive_time, Utc)
}

pub fn maybe_date_time(i: &[u8]) -> IResult<&[u8], Option<DateTime<Utc>>> {
    let (i, present) = boolean(i)?;
    if present {
        let (i, ts) = map(be_u64, unpack_timestamp)(i)?;
        Ok((i, Some(ts)))
    } else {
        Ok((i, None))
    }
}

pub fn date_time(i: &[u8]) -> IResult<&[u8], DateTime<Utc>> {
    map_res(maybe_date_time, |dt: Option<DateTime<Utc>>| {
        dt.ok_or("DateTime may not be null")
    })(i)
}

fn decode_version(s: &[u8]) -> Result<usize, ()> {
    match str::from_utf8(s) {
        Ok(s) => s.parse().map_err(|_| ()),
        Err(_) => Err(()),
    }
}

pub fn version_header<'a>(prefix: &'static [u8]) -> impl FnMut(&'a[u8]) -> IResult<&'a [u8], usize> {
    preceded(tag(prefix), map_res(take(3usize), decode_version))
}

pub fn compression_type(i: &[u8]) -> IResult<&[u8], CompressionType> {
    map_res(be_u32,
        |x| {
            CompressionType::try_from(x).
                map_err(|e| format!("Invalid compression type: {}", e))
        }
    )(i)
}

pub fn unwrap_compression_type(
    flag: Option<bool>,
    compression_type: Option<CompressionType>,
) -> CompressionType {
    flag.map(|c| {
        if c {
            CompressionType::GZip
        } else {
            CompressionType::None
        }
    })
    .or(compression_type)
    .unwrap_or(CompressionType::None)
}

pub fn sha_string(i: &[u8]) -> IResult<&[u8], SHA1> {
    map_res(non_null_string, |s: String| SHA1::try_from(&s[..]))(i)
}

pub fn maybe_sha_string(i: &[u8]) -> IResult<&[u8], Option<SHA1>> {
    map_res(maybe_string, |text: Option<String>| {
        if let Some(s) = text {
            SHA1::try_from(&s[..]).map(Some)
        } else {
            Ok(None)
        }
    })(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn non_null_string_is_parsed() {
        let data = &[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x2c, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21,
        ];
        match maybe_string(data) {
            Ok((_, s)) => assert_eq!(s.unwrap(), "Hello, world!"),
            Err(e) => assert!(false, "Parse failed: {}", e),
        }
    }

    #[test]
    fn null_string_is_an_error() {
        let data = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let r = non_null_string(data);
        assert!(r.is_err());
    }

    #[test]
    fn null_maybe_string_is_not_an_error() {
        let data = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        match maybe_string(data) {
            Ok((_, r)) => assert_eq!(r, None),
            Err(e) => assert!(false, "Parse failed: {}", e),
        }
    }

    #[test]
    fn non_null_datetime_is_parsed() {
        let data = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        match date_time(data) {
            Ok((_, dt)) => {
                let expected =
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 256000), Utc);
                assert_eq!(dt, expected);
            }
            Err(e) => assert!(false, "Parse failed: {}", e),
        }
    }

    #[test]
    fn null_datetime_is_an_error() {
        let data = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        let r = date_time(data);
        assert!(r.is_err());
    }

    #[test]
    fn null_maybe_datetime_is_not_an_error() {
        let data = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        match maybe_date_time(data) {
            Ok((_, r)) => assert_eq!(r, None),
            Err(e) => assert!(false, "Parse failed: {}", e),
        }
    }
}
