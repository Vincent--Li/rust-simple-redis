use anyhow::Result;

use crate::{
    cmd::{extract_args, validate_command, CommandError, HGet, HGetAll, HSet},
    Array, RespFrame,
};

impl TryFrom<Array> for HGet {
    type Error = CommandError;

    fn try_from(_value: Array) -> Result<Self, Self::Error> {
        validate_command(&_value, &["hget"], 2)?;

        let mut args = extract_args(_value, 1)?.into_iter();

        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl TryFrom<Array> for HGetAll {
    type Error = CommandError;

    fn try_from(_value: Array) -> Result<Self, Self::Error> {
        validate_command(&_value, &["hgetall"], 1)?;

        let mut args = extract_args(_value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArguments("Invalid key".to_string())),
        }
    }
}

impl TryFrom<Array> for HSet {
    type Error = CommandError;

    fn try_from(_value: Array) -> Result<Self, Self::Error> {
        validate_command(&_value, &["hset"], 3)?;

        let mut args = extract_args(_value, 1)?.into_iter();

        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::RespDecode;

    use super::*;

    #[test]
    fn test_hget_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nkey\r\n$5\r\nfield\r\n");
        let frame = Array::decode(&mut buf)?;

        let hget_command: HGet = HGet::try_from(frame)?;
        assert_eq!(hget_command.key, "key");
        assert_eq!(hget_command.field, "field");
        Ok(())
    }

    #[test]
    fn test_hset_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n");
        let frame = Array::decode(&mut buf)?;

        let hset_command: HSet = HSet::try_from(frame)?;

        assert_eq!(hset_command.key, "key");
        assert_eq!(hset_command.field, "field");
        assert_eq!(hset_command.value, RespFrame::BulkString("value".into()));
        Ok(())
    }

    #[test]
    fn test_hgetall_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nkey\r\n");
        let frame = Array::decode(&mut buf)?;

        let hgetall_command = HGetAll::try_from(frame)?;
        assert_eq!(hgetall_command.key, "key");
        Ok(())
    }
}
