use crate::{
    cmd::{extract_args, validate_command},
    Array, RespFrame,
};

use super::{CommandError, Get};

impl TryFrom<Array> for Get {
    type Error = CommandError;

    fn try_from(value: Array) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;

        let args = extract_args(&value, 1)?;
        match args[0] {
            RespFrame::BulkString(ref key) => Ok(Get {
                key: String::from_utf8_lossy(key).to_string(),
            }),
            _ => Err(CommandError::InvalidArguments("invalid key".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cmd::Get, Array, BulkString, RespFrame};

    #[test]
    fn test_get_command() {
        let cmd = Array::new(vec![
            RespFrame::BulkString(BulkString(b"get".to_vec())),
            RespFrame::BulkString(BulkString(b"key".to_vec())),
        ]);
        let get = Get::try_from(cmd).unwrap();
        assert_eq!("key", get.key);
    }
}
