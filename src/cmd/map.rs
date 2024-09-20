use anyhow::Result;

use crate::{
    cmd::{extract_args, validate_command},
    Array, RespFrame,
};

use super::{CommandError, Get, Set};

impl TryFrom<Array> for Get {
    type Error = CommandError;

    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;

        let args = extract_args(value, 1)?;
        match &args[0] {
            RespFrame::BulkString(key) => Ok(Get {
                key: String::from_utf8(key.0.clone())?,
            }),
            _ => Err(CommandError::InvalidArguments("invalid key".to_string())),
        }
    }
}

impl TryFrom<Array> for Set {
    type Error = CommandError;

    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();

        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(Set {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidArguments(
                "invalid key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::{cmd::Get, Array, RespDecode, RespFrame};

    #[test]
    fn test_get_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = Array::decode(&mut buf)?;
        let get = Get::try_from(frame)?;
        assert_eq!("hello", get.key);
        Ok(())
    }

    #[test]
    fn test_set_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = Array::decode(&mut buf)?;
        let set = super::Set::try_from(frame)?;
        assert_eq!("hello", set.key);
        assert_eq!(RespFrame::BulkString("world".into()), set.value);
        Ok(())
    }
}
