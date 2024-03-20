use std::fmt;

use chrono::{DateTime, Utc};

pub enum CommandType {
    Get(Get),
    Set(Set),
    Echo(String),
    Ping,
    Info(Info),
}

pub struct Get {
    pub key: String,
}

pub enum Ttl {
    Ex(DateTime<Utc>),
    Px(DateTime<Utc>),
    Exat(DateTime<Utc>),
    Pxat(DateTime<Utc>),
    Keepttl,
}

pub enum SetFlag {
    Nx,
    Xx,
}

pub struct Set {
    pub key: String,
    pub value: String,
    pub ttl: Option<Ttl>,
    pub flag: Option<SetFlag>,
    pub get: bool,
}

pub enum InfoSelection {
    // Server,
    // Clients,
    // Memory,
    // Persistence,
    // Stats,
    Replication,
    // Cpu,
    // CommandStats,
    // LatencyStats,
    // Sentinel,
    // Cluster,
    // Modules,
    // Keyspace,
    // ErrorStats,
    Unimplimented,
}

pub struct Info {
    pub section: Option<Vec<InfoSelection>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DecodeError {
    Unimplimented,
    SubUnimplimented,
    InvalidCommand,
    InvalidOption,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid input for decode")
    }
}

pub fn decode(mut input: Vec<String>) -> Result<CommandType, DecodeError> {
    let command = input.first().to_owned().unwrap().as_str();
    match command {
        "get" => {
            if input.len() == 2 {
                Ok(CommandType::Get(Get {
                    key: (input.get(1).unwrap()).to_string(),
                }))
            } else {
                Err(DecodeError::InvalidCommand)
            }
        }
        "set" => {
            if input.len() < 3 {
                return Err(DecodeError::InvalidOption);
            }

            let key = input.get(1).unwrap().to_string();
            let value = input.get(2).unwrap().to_string();
            let mut ttl = None;
            let mut flag = None;
            let mut get = false;

            input = input.split_off(3);

            let mut options = input.iter();

            while let Some(opt) = options.next() {
                match opt.to_ascii_lowercase().as_str() {
                    "nx" => {
                        if flag.is_none() && !get && ttl.is_none() {
                            flag = Some(SetFlag::Nx)
                        } else {
                            return Err(DecodeError::InvalidOption);
                        }
                    }
                    "xx" => {
                        if flag.is_none() && !get && ttl.is_none() {
                            flag = Some(SetFlag::Xx)
                        } else {
                            return Err(DecodeError::InvalidOption);
                        }
                    }
                    "get" => {
                        if ttl.is_none() {
                            get = true;
                        } else {
                            return Err(DecodeError::InvalidOption);
                        }
                    }
                    "ex" => {
                        ttl = Some(Ttl::Ex(
                            DateTime::checked_add_signed(
                                Utc::now(),
                                chrono::TimeDelta::try_seconds(
                                    options.next().unwrap().parse::<i64>().unwrap(),
                                )
                                .unwrap_or_default(),
                            )
                            .unwrap(),
                        ))
                    }
                    "px" => {
                        ttl = Some(Ttl::Px(
                            DateTime::checked_add_signed(
                                Utc::now(),
                                chrono::TimeDelta::try_milliseconds(
                                    options.next().unwrap().parse::<i64>().unwrap(),
                                )
                                .unwrap_or_default(),
                            )
                            .unwrap(),
                        ))
                    }
                    "exat" => {
                        ttl = Some(Ttl::Exat(
                            DateTime::from_timestamp_millis(
                                options.next().unwrap().parse::<i64>().unwrap() * 1000,
                            )
                            .unwrap(),
                        ))
                    }
                    "pxat" => {
                        ttl = Some(Ttl::Pxat(
                            DateTime::from_timestamp_millis(
                                options.next().unwrap().parse::<i64>().unwrap(),
                            )
                            .unwrap(),
                        ))
                    }
                    "keepttl" => {}
                    _ => return Err(DecodeError::InvalidOption),
                }
            }

            Ok(CommandType::Set(Set {
                key,
                value,
                ttl,
                flag,
                get,
            }))
        }
        "echo" => {
            if input.len() > 1 {
                return Ok(CommandType::Echo(input.get(1).unwrap().to_string()));
            }
            Ok(CommandType::Echo(String::from("")))
        }
        "info" => {
            if input.len() < 2 {
                return Ok(CommandType::Info(Info { section: None }));
            }

            let mut out = Vec::new();

            for opt in input {
                match opt.to_ascii_lowercase().as_str() {
                    "replication" => out.push(InfoSelection::Replication),
                    _ => out.push(InfoSelection::Unimplimented),
                }
            }

            Ok(CommandType::Info(Info { section: Some(out) }))
        }
        "ping" => Ok(CommandType::Ping),
        _ => Err(DecodeError::Unimplimented),
    }
}
