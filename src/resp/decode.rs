use super::{RespDecode, RespError, RespFrame, SimpleString};
use bytes::BytesMut;

impl RespDecode for RespFrame {
    fn decode(buf: BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                todo!()
            }
            _ => todo!(),
        }
    }
}

impl RespDecode for SimpleString {
    fn decode(buf: BytesMut) -> Result<Self, RespError> {
        if buf.len() < 3 {
            return Err(RespError::NotComplete);
        }

        if !buf.starts_with(b"+") {
            return Err(RespError::InvalidFrameType(format!(
                "expected SimpleString(+)' but got {:?}",
                buf
            )));
        }

        // search for \r\n
        todo!()
    }
}
