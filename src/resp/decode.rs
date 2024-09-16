use super::{RespDecode, RespFrame, SimpleString};
use anyhow::Result;

impl RespDecode for SimpleString {
    fn decode(_buf: Self) -> Result<RespFrame> {
        todo!()
    }
}
