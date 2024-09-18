/*
- simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - big number: "([+|-]<number>\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
    - ...
*/
mod decode;
mod encode;

use anyhow::Result;
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

#[enum_dispatch]
pub trait RespDecode: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch(RespEncode)]
// #[enum_dispatch(RespDecode)]
#[derive(Debug, PartialEq, PartialOrd)]
// #[enum_dispatch(RespDecode)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    BulkError(BulkError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(NullBulkString),
    Array(Array),
    NullArray(NullArray),
    Null(Null),
    Boolean(bool),
    Double(f64),
    Map(Map),
    Set(Set),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct SimpleString(String);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct SimpleError(String);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct NullArray;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Null;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct NullBulkString;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Array(Vec<RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Map(BTreeMap<String, RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Set(Vec<RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct BulkError(String);
#[derive(Debug, PartialEq, PartialOrd)]
// when encounter struct wrapper, we could impl Deref to access inner value as if it is the inner type
pub struct BulkString(Vec<u8>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Array {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Map {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Set {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Set {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BulkError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl Array {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        Array(v.into())
    }
}

impl Map {
    pub fn new(m: BTreeMap<String, RespFrame>) -> Self {
        Map(m)
    }
}

impl Set {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        Set(s.into())
    }
}

impl BulkError {
    pub fn new(s: impl Into<String>) -> Self {
        BulkError(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.into())
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.into())
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.into())
    }
}

impl From<&str> for BulkError {
    fn from(s: &str) -> Self {
        BulkError(s.into())
    }
}
