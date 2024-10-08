use std::collections::BTreeMap;

use super::{
    Array, BulkError, BulkString, Map, Null, NullArray, NullBulkString, RespDecode, RespError,
    RespFrame, Set, SimpleError, SimpleString,
};
use anyhow::Result;
use bytes::{Buf, BytesMut};

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

impl RespDecode for RespFrame {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => {
                // try null array first
                match NullArray::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = Array::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                // try null bulk string first
                match NullBulkString::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = BulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'~') => {
                let frame = Set::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = Map::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'!') => {
                let frame = BulkError::decode(buf)?;
                Ok(frame.into())
            }
            _ => Err(RespError::InvalidFrame("invalid frame type".into())),
        }
    }
}

impl RespDecode for SimpleString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "+";
        let end = extract_simple_frame_data(buf, prefix)?;

        // split the buffer
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);

        Ok(SimpleString::new(s.to_string()))
    }
}

impl RespDecode for SimpleError {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "-";
        let end = extract_simple_frame_data(buf, prefix)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);

        Ok(SimpleError::new(s.to_string()))
    }
}

impl RespDecode for BulkError {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "!";
        let end = extract_simple_frame_data(buf, prefix)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[prefix.len()..end]);
        Ok(BulkError::new(s.to_string()))
    }
}

impl RespDecode for Null {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "Null")?;
        Ok(Null)
    }
}

impl RespDecode for NullArray {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
        Ok(NullArray)
    }
}

impl RespDecode for NullBulkString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(NullBulkString)
    }
}

impl RespDecode for i64 {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = ":";
        let end = extract_simple_frame_data(buf, prefix)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[prefix.len()..end]);

        Ok(s.parse()?)
    }
}

impl RespDecode for bool {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }
}

impl RespDecode for BulkString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, "$")?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }
}

impl RespDecode for Array {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "*";
        let (end, len) = parse_length(buf, prefix)?;
        let total_len = cal_total_length(buf, len, prefix)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut array = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            array.push(frame);
        }
        Ok(Array::new(array))
    }
}

impl RespDecode for f64 {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = ",";
        let end = extract_simple_frame_data(buf, prefix)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[prefix.len()..end]);
        Ok(s.parse()?)
    }
}

impl RespDecode for Map {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "*";
        let (end, len) = parse_length(buf, prefix)?;
        let total_len = cal_total_length(buf, len, prefix)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Map::new(BTreeMap::new());
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }

        Ok(frames)
    }
}

impl RespDecode for Set {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let prefix = "~";
        let (end, len) = parse_length(buf, prefix)?;
        let total_len = cal_total_length(buf, len, prefix)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Set::new(Vec::new());
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }

        Ok(frames)
    }
}

#[allow(dead_code)]
fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got: {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());
    Ok(())
}

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expected SimpleString(+)' but got {:?}",
            buf
        )));
    }

    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;
    Ok(end)
}

fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }

    None
}

fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end, s.parse()?))
}

fn cal_total_length(buf: &[u8], len: usize, prefix: &str) -> Result<usize, RespError> {
    let data = &buf[len + CRLF_LEN..];
    match prefix {
        "*" | "~" => find_crlf(data, len)
            .map(|end| len + CRLF_LEN + end)
            .ok_or(RespError::NotComplete),
        "%" => find_crlf(data, len * 2)
            .map(|end| len + CRLF_LEN + end)
            .ok_or(RespError::NotComplete),
        _ => Ok(len + CRLF_LEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()));

        buf.extend_from_slice(b"+hello\r");
        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret, Err(RespError::NotComplete));

        buf.extend_from_slice(b"\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("hello".to_string()));

        Ok(())
    }

    #[test]
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-ERR unknown command 'hello'\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(
            frame,
            SimpleError::new("ERR unknown command 'hello'".to_string())
        );

        buf.extend_from_slice(b"-ERR unknown command 'hello'");
        let ret = SimpleError::decode(&mut buf);
        assert_eq!(ret, Err(RespError::NotComplete));
        Ok(())
    }

    #[test]
    fn test_f64_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",3.14768\r\n");

        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 3.14768);

        buf.extend_from_slice(b",3.14768");
        let ret = f64::decode(&mut buf);
        assert_eq!(ret, Err(RespError::NotComplete));
        Ok(())
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$5\r\nhello\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

        let frame = Array::decode(&mut buf)?;
        assert_eq!(
            frame,
            Array::new(vec![
                RespFrame::BulkString(BulkString::new("hello")),
                RespFrame::BulkString(BulkString::new("foo")),
                RespFrame::BulkString(BulkString::new("bar"))
            ])
        );
        Ok(())
    }

    #[test]
    fn test_set_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

        let frame = Set::decode(&mut buf)?;
        assert_eq!(
            frame,
            Set::new(vec![
                RespFrame::BulkString(BulkString::new("foo")),
                RespFrame::BulkString(BulkString::new("bar"))
            ])
        );

        Ok(())
    }
}
