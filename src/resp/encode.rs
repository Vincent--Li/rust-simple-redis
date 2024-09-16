use super::{
    Array, BulkError, BulkString, Map, Null, NullArray, NullBulkString, RespEncode, Set,
    SimpleError, SimpleString,
};

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

const BUF_CAP: usize = 4096;

// - simple string: "+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

// - error: "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

// - bulk error: "!<length>\r\n<error>\r\n"
impl RespEncode for BulkError {
    fn encode(self) -> Vec<u8> {
        format!("!{}\r\n{}\r\n", self.len(), self.0).into_bytes()
    }
}

// - integer: ":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self >= 0 { "+" } else { "-" };

        format!("{}{}\r\n", sign, self.abs()).into_bytes()
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncode for NullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

//     - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n
impl RespEncode for Array {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - null array: "*-1\r\n"
impl RespEncode for NullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

// - null: "_\r\n"
impl RespEncode for Null {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

// - boolean: "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self >= 0.0 { "+" } else { "-" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

// - map : "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for Map {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

// - set : "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for Set {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for element in self.0 {
            buf.extend_from_slice(&element.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString::new("OK").into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"+OK\r\n".to_vec());
        let frame: RespFrame = SimpleString::new("").into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"+\r\n".to_vec());
        let frame: RespFrame = SimpleString::new("+OK").into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"++OK\r\n".to_vec());
    }

    #[test]
    fn test_encode_simple_error() {
        let frame: RespFrame = SimpleError::new("Error message").into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"-Error message\r\n".to_vec());
    }
    #[test]
    fn test_encode_bulk_error() {
        let err_msg = "Error message";
        let frame: RespFrame = BulkError::new(err_msg).into();
        let encoded = frame.encode();
        assert_eq!(
            encoded,
            format!("!{}\r\n{}\r\n", err_msg.len(), err_msg).into_bytes()
        );
    }
    #[test]
    fn test_encode_integer() {
        let frame: RespFrame = 42.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"+42\r\n".to_vec());
        let frame: RespFrame = (-42).into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"-42\r\n".to_vec());
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new("Hello, world!".to_string()).into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"$13\r\nHello, world!\r\n".to_vec());
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = NullBulkString.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"$-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_array() {
        let frame: RespFrame = Array::new(vec![
            SimpleString::new("get").into(),
            BulkString::new("hello".to_string()).into(),
        ])
        .into();
        let encoded = frame.encode();
        println!(
            "test encode array {}",
            String::from_utf8(encoded.clone()).unwrap()
        );
        assert_eq!(encoded, b"*2\r\n+get\r\n$5\r\nhello\r\n".to_vec());
    }

    #[test]
    fn test_encode_null_array() {
        let frame: RespFrame = NullArray.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"*-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_null() {
        let frame: RespFrame = Null.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"_\r\n".to_vec());
    }

    #[test]
    fn test_encode_boolean() {
        let frame: RespFrame = true.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b"#t\r\n".to_vec());
    }

    #[test]
    fn test_encode_double() {
        let frame: RespFrame = 3.147.into();
        let encoded = frame.encode();
        assert_eq!(encoded, b",+3.147\r\n".to_vec());
    }

    #[test]
    fn test_encode_map() {
        let pairs = vec![
            ("key1".to_string(), SimpleString::new("value1").into()),
            (
                "key2".to_string(),
                BulkString::new("value2".to_string()).into(),
            ),
        ];
        let frame: RespFrame = Map::new(BTreeMap::from_iter(pairs)).into();
        let encoded = frame.encode();
        println!(
            "test encode map {}",
            String::from_utf8(encoded.clone()).unwrap()
        );
        assert_eq!(
            encoded,
            b"%2\r\n+key1\r\n+value1\r\n+key2\r\n$6\r\nvalue2\r\n".to_vec()
        );
    }

    #[test]
    fn test_encode_set() {
        let values = vec![
            SimpleString::new("value1").into(),
            BulkString::new("value2".to_string()).into(),
        ];

        let frame: RespFrame = Set::new(values).into();
        let encoded = frame.encode();
        println!(
            "test encode set {}",
            String::from_utf8(encoded.clone()).unwrap()
        );
        assert_eq!(encoded, b"~2\r\n+value1\r\n$6\r\nvalue2\r\n".to_vec());
    }
}
