use crate::Array;

use super::{CommandError, HGet};

impl TryFrom<Array> for HGet {
    type Error = CommandError;

    fn try_from(_value: Array) -> Result<Self, Self::Error> {
        todo!()
    }
}
