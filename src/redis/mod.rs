pub mod decoder;
pub mod encoder;
pub mod handle_input;
pub mod set_parse;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Data {
    pub value: String,
    pub expires_at: Option<DateTime<chrono::Utc>>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            value: String::new(),
            expires_at: Some(chrono::Utc::now()),
        }
    }
}

impl Data {
    pub fn new(data: String, expires: Option<DateTime<Utc>>) -> Self {
        Data {
            value: data,
            expires_at: expires,
        }
    }
}

pub enum RedisType {
    SimpleString,
    SimpleError,
    Integer,
    BulkString,
    Array,
    Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
}

pub fn get_redis_type(input: &str) -> Option<RedisType> {
    match input.chars().next().unwrap_or(' ') {
        '+' => Some(RedisType::SimpleString),
        '-' => Some(RedisType::SimpleError),
        ':' => Some(RedisType::Integer),
        '$' => Some(RedisType::BulkString),
        '*' => Some(RedisType::Array),
        '_' => Some(RedisType::Null),
        '#' => Some(RedisType::Boolean),
        ',' => Some(RedisType::Double),
        '(' => Some(RedisType::BigNumber),
        '!' => Some(RedisType::BulkError),
        '=' => Some(RedisType::VerbatimString),
        '%' => Some(RedisType::Map),
        '~' => Some(RedisType::Set),
        '>' => Some(RedisType::Push),
        _ => None,
    }
}
