// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let data_store = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

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
    data_store: Arc<Mutex<HashMap<String, String>>>,
) {
    let mut input_str = String::new();
    let _ = buf.read_line(&mut input_str);

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

    let input_type = input_type.unwrap();

    let mut inputs: Vec<String> = Vec::new();

    match input_type {
        RedisType::Array => {
            // println!("{}", input_str);
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
                            let (_, len) = input_str.split_at(1);
                            let len = len.trim();
                            let len = len.parse::<usize>();
                            let mut temp = String::new();
                            let _ = buf.read_line(&mut temp).unwrap();
                            // temp = temp.trim().to_owned();
                            if len.is_err() {
                                let _ =
                                    stream.write(b"-ERR invalid input - no length\r\n").unwrap();
                                return;
                            };
                            let len = len.unwrap();
                            if len == temp.len() {
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
                let data_store = parse_inputs(inputs, stream, data_store);
                read_input(buf, stream, data_store);
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
    data_store: Arc<Mutex<HashMap<String, String>>>,
) -> Arc<Mutex<HashMap<String, String>>> {
    if inputs.is_empty() {
        let _ = stream.write(b"-ERR invalid commands\r\n");
        return data_store;
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
                let _ = &stream.write(b"_\r\n");
                return data_store;
            }
            let msg = args.first().unwrap();
            let data = format!("${}\r\n{}\r\n", msg.len(), msg);
            println!("{}", data);
            let _ = &stream.write(data.as_bytes()).unwrap();
        }
        "set" => {
            if args.len() < 2 {
                let _ = &stream.write(b"-ERR incorrect arguments");
                return data_store;
            }
            let mut store = data_store.lock().unwrap();
            store.insert(
                args.first().unwrap().to_string(),
                args.get(1).unwrap().to_string(),
            );
            let _ = &stream.write(b"+OK\r\n").unwrap();
        }
        "get" => {
            if args.is_empty() {
                let _ = &stream.write(b"-ERR incorrect arguments");
                return data_store;
            }
            let store = data_store.lock().unwrap();
            let value = store.get(args.first().unwrap());
            if let Some(value) = value {
                let data = format!("${}\r\n{}\r\n", value.len(), value);
                let _ = &stream.write(data.as_bytes());
            } else {
                let _ = &stream.write(b"$-1\r\n");
            }
        }
        _ => {
            println!("err: {}", command);
            // let _ = &stream.write(b"-Unknown Command").unwrap();
        }
    };
    data_store
}
