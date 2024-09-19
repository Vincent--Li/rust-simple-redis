mod hmap;
mod map;

use crate::{Array, RespError, RespFrame};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command {0}")]
    InvalidCommand(String),
    #[error("invalid arguments {0}")]
    InvalidArguments(String),
    #[error("{0}")]
    RespError(#[from] RespError),
}

pub trait CommandExecutor {
    fn execute(&self, cmd: Command) -> RespFrame;
}

pub enum Command {
    Set(Set),
    Get(Get),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
}

#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub value: RespFrame,
}

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

#[derive(Debug)]
pub struct HGet {
    pub key: String,
    pub field: String,
}

#[derive(Debug)]
pub struct HSet {
    pub key: String,
    pub field: String,
    pub value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    pub key: String,
}

impl TryFrom<Array> for Command {
    type Error = CommandError;

    fn try_from(_value: Array) -> Result<Self, Self::Error> {
        todo!()
    }
}

fn validate_command(
    value: &Array,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArguments(format!(
            " invalid number of arguments for command {}, must have exact {}",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref bs) => {
                if bs.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "invalid command name {:?}",
                        bs
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidArguments(
                    "invalid command arguments".to_string(),
                ))
            }
        }
    }

    Ok(())
}

fn extract_args(value: &Array, start: usize) -> Result<Vec<&RespFrame>, CommandError> {
    Ok(value.iter().skip(start).collect::<Vec<&RespFrame>>())
}
