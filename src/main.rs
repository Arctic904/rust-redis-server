// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut bufreader = BufReader::new(stream.try_clone().unwrap());
                read_input(&mut bufreader, &mut stream)
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
    match input.chars().nth(0).unwrap_or(' ') {
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

pub fn read_input(buf: &mut BufReader<TcpStream>, stream: &mut TcpStream) {
    let mut input_str = String::new();
    let _ = buf.read_line(&mut input_str);

    let input_str = input_str.trim();

    let input_type = get_redis_type(&input_str);

    if input_type.is_none() {
        let _ = stream.write(b"-Error Unknown Input\r\n").unwrap();
        return;
    }

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
                            let (_, len) = input_str.split_at(1);
                            let len = len.trim();
                            let len = len.parse::<usize>();
                            let mut temp = String::new();
                            let _ = buf.read_line(&mut temp).unwrap();
                            // temp = temp.trim().to_owned();
                            if let Err(_) = len {
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
                            println!("{}", temp);
                            inputs.push(temp);
                        }
                        _ => {
                            let _ = stream.write(b"-ERR invalid input type\r\n").unwrap();
                        }
                    }
                    i += 1;
                }
                parse_inputs(inputs, stream)
            } else {
                let _ = stream.write(b"-ERR invalid input length\r\n").unwrap();
            }
        }
        _ => {
            let _ = stream.write(b"-ERR Unexpected input type\r\n").unwrap();
        }
    }

    // buf.lines().for_each(|line| {
    //     let binding = line.unwrap_or("".to_owned());
    //     let line = binding.as_str();
    //     println!("{}", line);
    //     match line {
    //         "ping" => {
    //             let _ = &stream.write(b"+PONG\r\n").unwrap();
    //         }
    //         "echo" => {
    //             let _ = &stream.write(b"+test").unwrap();
    //         }
    //         _ => {
    //             println!("err: {}", line);
    //             // let _ = &stream.write(b"-Unknown Command").unwrap();
    //         } //
    //     };
    // })
}

pub fn parse_inputs(mut inputs: Vec<String>, stream: &mut TcpStream) {
    if inputs.len() == 0 {
        let _ = stream.write(b"-ERR invalid commands\r\n");
    }
    let binding = inputs.get(0).unwrap().to_ascii_lowercase();
    let command = binding.as_str().trim();
    let args = inputs.split_off(1);

    match command {
        "ping" => {
            let _ = &stream.write(b"+PONG\r\n").unwrap();
        }
        "echo" => {
            let msg = args.get(0).unwrap();
            let data = format!("${}\r\n{}\r\n", msg.len(), msg);
            println!("{}", data);
            let _ = &stream.write(data.as_bytes()).unwrap();
        }
        _ => {
            println!("err: {}", command);
            // let _ = &stream.write(b"-Unknown Command").unwrap();
        }
    };
}
