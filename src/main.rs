// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use clap::Parser;

const DEFAULT_DT: DateTime<Utc> = DateTime::from_timestamp_nanos(0);
const NULL_REPLY: &[u8; 5] = b"$-1\r\n";

#[derive(Debug, Parser)]
#[clap(name = "redis-server", version)]
pub struct RedisServer {
    #[clap(long, default_value_t = 6379)]
    port: u16,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args = RedisServer::parse();

    println!("Given Port: {}", args.port);

    let data_store = Arc::new(Mutex::new(HashMap::<String, Data>::new()));

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).unwrap();

    for stream in listener.incoming() {
        let cloned_store = Arc::clone(&data_store);
        let _test = thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut bufreader = BufReader::new(stream.try_clone().unwrap());
                read_input(&mut bufreader, &mut stream, cloned_store);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    value: String,
    expires_at: DateTime<chrono::Utc>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            value: String::new(),
            expires_at: chrono::Utc::now(),
        }
    }
}

impl Data {
    pub fn new(data: String, expires: chrono::DateTime<chrono::Utc>) -> Self {
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

pub fn read_input(
    buf: &mut BufReader<TcpStream>,
    stream: &mut TcpStream,
    data_store: Arc<Mutex<HashMap<String, Data>>>,
) {
    let mut input_str = String::new();
    let test = buf.read_line(&mut input_str);

    if let Err(test) = test {
        println!("err: {}", test);
    }

    let input_str = input_str.trim();

    if input_str.is_empty() {
        return;
    }

    // println!("{}", input_str);

    let input_type = get_redis_type(&input_str);

    if input_type.is_none() {
        let _ = stream.write(b"-Error Unknown Input\r\n").unwrap();
        return;
    }

    // println!("{}", &input_str);

    let input_type = input_type.unwrap();

    let mut inputs: Vec<String> = Vec::new();

    match input_type {
        RedisType::Array => {
            println!("{}", input_str);
            let (_, len) = input_str.split_at(1);
            // println!("len: {}", len);
            let len = len.trim();
            let len = len.parse::<usize>();
            if let Ok(len) = len {
                let mut i = 0;
                while i < len {
                    let mut buf_type = String::new();
                    let _ = buf.read_line(&mut buf_type).unwrap();
                    let temp = get_redis_type(&buf_type).unwrap_or(RedisType::SimpleError);
                    match temp {
                        RedisType::BulkString => {
                            let (_old, len) = buf_type.split_at(1);
                            let len = len.trim();
                            let len = len.parse::<usize>();
                            let mut temp = String::new();
                            let _ = buf.read_line(&mut temp).unwrap();
                            temp = temp.trim().to_owned();
                            if len.is_err() {
                                let _ =
                                    stream.write(b"-ERR invalid input - no length\r\n").unwrap();
                                return;
                            };
                            let len = len.unwrap();
                            // println!(
                            //     "{}{}\n{}\n{:?}: {}",
                            //     old,
                            //     len,
                            //     temp.len(),
                            //     temp.as_bytes(),
                            //     temp
                            // );
                            if len != temp.len() {
                                let _ = stream
                                    .write(b"-ERR invalid input - incorrect length\r\n")
                                    .unwrap();
                                return;
                            }
                            temp = temp.trim().to_owned();
                            // println!("{}", temp);
                            inputs.push(temp);
                        }
                        _ => {
                            let _ = stream.write(b"-ERR invalid input type\r\n").unwrap();
                        }
                    }
                    i += 1;
                }
                parse_inputs(inputs, stream, Arc::clone(&data_store));
                read_input(buf, stream, Arc::clone(&data_store));
            } else {
                let _ = stream.write(b"-ERR invalid input length\r\n").unwrap();
            }
        }
        _ => {
            let _ = stream.write(b"-ERR Unexpected input type\r\n").unwrap();
        }
    }
    stream.flush().unwrap();
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

pub fn parse_inputs(
    mut inputs: Vec<String>,
    stream: &mut TcpStream,
    data_store: Arc<Mutex<HashMap<String, Data>>>,
) {
    if inputs.is_empty() {
        let _ = stream.write(b"-ERR invalid commands\r\n");
        return;
    }
    let binding = inputs.first().unwrap().to_ascii_lowercase();
    let command = binding.as_str().trim();
    let args = inputs.split_off(1);

    match command {
        "ping" => {
            let _ = &stream.write(b"+PONG\r\n").unwrap();
        }
        "echo" => {
            if args.first().is_none() {
                let _ = &stream.write(NULL_REPLY);
                return;
            }
            let msg = args.first().unwrap();
            let data = format!("${}\r\n{}\r\n", msg.len(), msg);
            println!("{}", data);
            let _ = &stream.write(data.as_bytes()).unwrap();
        }
        "set" => {
            if args.len() < 2 {
                let _ = &stream.write(b"-ERR incorrect arguments");
                return;
            }
            let mut store = data_store.lock().unwrap();
            if args.len() > 2 {
                let mut args = args.iter();
                let key = args.next().unwrap().to_string();
                let value = args.next().unwrap().to_string();
                let flag_check: Vec<String> =
                    args.clone().map(|x| x.to_ascii_lowercase()).collect();
                let nxflag = flag_check.contains(&"nx".to_string());
                let xxflag = flag_check.contains(&"xx".to_string());
                let getflag = flag_check.contains(&"get".to_string());
                let mut expires = false;
                let mut keepttl = false;
                let mut exflag = String::from("");

                for x in args.by_ref() {
                    let x = x.to_ascii_lowercase();
                    if ["ex", "px", "exat", "pxat", "keepttl"].contains(&x.as_str()) {
                        expires = true;
                        if x == "keepttl" {
                            keepttl = true;
                        }
                        exflag = x;
                        break;
                    }
                }

                if (store.contains_key(&key) && nxflag) || (!store.contains_key(&key) && xxflag) {
                    let _ = &stream.write(NULL_REPLY);
                    return;
                }

                let expiry: DateTime<Utc> = if expires && !keepttl {
                    match exflag.as_str() {
                        "ex" => DateTime::checked_add_signed(
                            Utc::now(),
                            chrono::TimeDelta::try_seconds(
                                args.next().unwrap().parse::<i64>().unwrap(),
                            )
                            .unwrap_or_default(),
                        )
                        .unwrap(),
                        "px" => DateTime::checked_add_signed(
                            Utc::now(),
                            chrono::TimeDelta::try_milliseconds(
                                args.next().unwrap().parse::<i64>().unwrap(),
                            )
                            .unwrap_or_default(),
                        )
                        .unwrap(),
                        "exat" => DateTime::from_timestamp_millis(
                            args.next().unwrap().parse::<i64>().unwrap() * 1000,
                        )
                        .unwrap(),
                        "pxat" => DateTime::from_timestamp_millis(
                            args.next().unwrap().parse::<i64>().unwrap(),
                        )
                        .unwrap(),
                        _ => {
                            let _ = &stream.write(b"-ERR improper date flag");
                            return;
                        }
                    }
                } else if expires {
                    store.get(&key).unwrap().expires_at
                } else {
                    DEFAULT_DT
                };

                println!(
                    "nx: {}\nxx: {}\nget: {}\nexpires: {}",
                    nxflag, xxflag, getflag, expiry
                );

                if nxflag && xxflag {
                    let _ = &stream.write(NULL_REPLY);
                    return;
                }

                let old = store.insert(key, Data::new(value, expiry));

                if let Some(old) = old {
                    if getflag {
                        let _ = &stream
                            .write(format!("${}\r\n{}\r\n", old.value.len(), old.value).as_bytes())
                            .unwrap();
                    }
                } else if getflag {
                    let _ = &stream.write(NULL_REPLY).unwrap();
                    return;
                }
                let _ = &stream.write(b"+OK\r\n").unwrap();
                return;
            }
            store.insert(
                args.first().unwrap().to_string(),
                Data::new(args.get(1).unwrap().to_string(), DEFAULT_DT),
            );
            let _ = &stream.write(b"+OK\r\n").unwrap();
        }
        "get" => {
            if args.is_empty() {
                let _ = &stream.write(b"-ERR incorrect arguments");
                return;
            }
            let store = data_store.lock().unwrap();
            let key = args.first().unwrap();
            let data = store.get(key);
            println!("{:?}\n{}\n{:?}", store.keys(), key, data);
            if let Some(data) = data {
                if data.expires_at == DEFAULT_DT {
                    let output = format!("${}\r\n{}\r\n", data.value.len(), data.value);
                    let _ = &stream.write(output.as_bytes());
                    return;
                }
                let diff = data.expires_at.signed_duration_since(Utc::now());
                if diff <= chrono::Duration::zero() {
                    let _ = &stream.write(NULL_REPLY);
                    return;
                }
                let output = format!("${}\r\n{}\r\n", data.value.len(), data.value);
                let _ = &stream.write(output.as_bytes());
            } else {
                println!("{}", args.first().unwrap());
                let _ = &stream.write(NULL_REPLY);
            }
        }
        _ => {
            println!("err: {}", command);
            // let _ = &stream.write(b"-Unknown Command").unwrap();
        }
    };
}
