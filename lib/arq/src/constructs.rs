use std::{
    convert::{TryFrom, TryInto},
    fmt::format,
    str,
};

use chrono::prelude::*;

use nom::{
    cond, do_parse, many_m_n, map, map_res, named,
    number::streaming::{be_u32, be_u64, be_u8},
};

use crate::{CompressionType, SHA1};

named!(
    pub boolean<bool>,
    map!(be_u8, |b| (b != 0))
);

named!(
    sized_string<String>,
    do_parse!(
        n: be_u64
            >> s: map_res!(many_m_n!(n as usize, n as usize, be_u8), String::from_utf8)
            >> (s)
    )
);

named!(
    pub maybe_string<Option<String>>,
    do_parse!(
        is_present: boolean
        >> s: cond!(is_present, sized_string) >> (s))
);

named!(
    pub non_null_string<String>,
    map_res!(maybe_string, |s: Option<String>| {
        s.ok_or("String may not be null")
    })
);

named!(
    pub sha1<SHA1>,
    map_res!(many_m_n!(20, 20, be_u8), |v: Vec<u8>| v.try_into())
);

fn unpack_timestamp(ms: u64) -> DateTime<Utc> {
    let s = (ms / 1000) as i64;
    let ns = ((ms % 1000) * 1000) as u32;
    let naive_time = NaiveDateTime::from_timestamp(s, ns);
    DateTime::<Utc>::from_utc(naive_time, Utc)
}

named!(
    pub maybe_date_time<Option<DateTime<Utc>>>,
    do_parse!(
        is_present: be_u8
        >> milliseconds: cond!(is_present != 0, be_u64)
        >> (milliseconds.map(unpack_timestamp))
    )
);

named!(
    pub date_time<DateTime<Utc>>,
    map_res!(maybe_date_time,
        |dt: Option<DateTime<Utc>>| dt.ok_or("DateTime may not be null"))
);

fn decode_version(s: &[u8]) -> Result<usize, ()> {
    match str::from_utf8(s) {
        Ok(s) => s.parse().map_err(|_| ()),
        Err(_) => Err(()),
    }
}

pub fn version_header<'a>(input: &'a [u8], prefix: &'static [u8]) -> nom::IResult<&'a [u8], usize> {
    use nom::{
        bytes::streaming::{tag, take},
        combinator::map_res,
        sequence::preceded,
    };

    preceded(tag(prefix), map_res(take(3usize), decode_version))(input)
}

named!(
    pub compression_type<CompressionType>,
    map_res!(
        be_u32,
        |x| {
            CompressionType::try_from(x).
                map_err(|e| format!("Invalid compression type: {}", e))
        }
    )
);

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

named!(
    pub sha_string<SHA1>,
    map_res!(non_null_string, |s: String| {
        SHA1::try_from(&s[..])
    })
);

named!(
    pub maybe_sha_string<Option<SHA1>>,
    map_res!(maybe_string, |text: Option<String>| {
        if let Some(s) = text {
            SHA1::try_from(&s[..]).map(Some)
        } else {
            Ok(None)
        }
    })
);

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
