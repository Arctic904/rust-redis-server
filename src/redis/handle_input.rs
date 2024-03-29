use super::decoder::CommandType::*;
use super::decoder::InfoSelection::*;
use super::set_parse::set_parse;
use super::{get_redis_type, RedisType};
use crate::{Context, NULL_REPLY};
use base64::prelude::*;
use chrono::Utc;
use std::borrow::BorrowMut;
use std::io::{BufRead, Write};

pub fn read_input(mut context: Context) {
    let mut bufreader = context.bufreader.borrow_mut().into_inner();
    let mut stream = context.stream.borrow_mut().into_inner();

    let mut input_str = String::new();
    let test = bufreader.read_line(&mut input_str);

    if let Err(test) = test {
        println!("err: {}", test);
    }

    let input_str = input_str.trim();

    if input_str.is_empty() {
        return;
    }

    // println!("{}", input_str);

    let input_type = get_redis_type(input_str);

    if input_type.is_none() {
        let _ = stream.write(b"-Error Unknown Input\r\n").unwrap();
        return;
    }

    // println!("{}", &input_str);

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
                    let temp = get_redis_type(&buf_type).unwrap_or(RedisType::SimpleError);
                    match temp {
                        RedisType::BulkString => {
                            let (_old, len) = buf_type.split_at(1);
                            let len = len.trim();
                            let len = len.parse::<usize>();
                            let mut temp = String::new();
                            let _ = bufreader.read_line(&mut temp).unwrap();
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

                parse_inputs(inputs, context.clone());
                read_input(context.clone());
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

pub fn parse_inputs(inputs: Vec<String>, mut context: Context) {
    let bufreader = context.bufreader.borrow_mut().into_inner();
    let mut stream = context.stream.borrow_mut().into_inner();

    if inputs.is_empty() {
        let _ = stream.write(b"-ERR invalid commands\r\n");
        return;
    }

    println!("{:?}", inputs);

    let command = super::decoder::decode(inputs);
    if let Err(err) = command {
        let _ = stream.write(format!("-ERR {}", err).as_bytes());
        return;
    }

    match command.unwrap() {
        Ping => {
            let _ = &stream.write(b"+PONG\r\n").unwrap();
        }
        Echo(val) => {
            if val.is_empty() {
                let _ = &stream.write(NULL_REPLY);
                return;
            }
            let data = format!("${}\r\n{}\r\n", val.len(), val);
            // println!("{}", data);
            let _ = &stream.write(data.as_bytes()).unwrap();
        }
        Set(set) => set_parse(set, &mut stream, context.store),
        Get(get) => {
            let mut store = context.store.lock().unwrap();
            let data = store.get(&get.key);
            // println!("{:?}\n{}\n{:?}", store.keys(), get.key, data);
            if let Some(data) = data {
                if data.expires_at.is_none() {
                    let output = format!("${}\r\n{}\r\n", data.value.len(), data.value);
                    let _ = &stream.write(output.as_bytes());
                    return;
                }
                let diff = data.expires_at.unwrap().signed_duration_since(Utc::now());
                if diff <= chrono::Duration::zero() {
                    let _ = &stream.write(NULL_REPLY);
                    store.remove(&get.key);
                    return;
                }
                let output = format!("${}\r\n{}\r\n", data.value.len(), data.value);
                let _ = &stream.write(output.as_bytes());
            } else {
                // println!("{}", get.key);
                let _ = &stream.write(NULL_REPLY);
            }
        }
        Info(info) => {
            //comment
            let role = match context.replica_conn {
                Some(_rep) => "slave".to_owned(),
                None => "master".to_owned(),
            };
            let output_str = info
                .section
                .unwrap()
                .iter()
                .map(|x| match x {
                    Replication => format!(
                        "# Replication
role:{}
connected_slaves:0
master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb
master_repl_offset:0
second_repl_offset:-1",
                        role
                    )
                    .to_owned(),
                    Unimplimented => "".to_owned(),
                })
                .collect::<Vec<String>>()
                .join("\r\n");
            let data = format!("${}\r\n{}\r\n", output_str.trim().len(), output_str.trim());
            let _ = &stream.write(data.as_bytes()).unwrap();
        }
        ReplConf(conf) => match conf {
            super::decoder::Conf::ListenPort(_port) => {
                let _ = &stream.write(b"+OK\r\n").unwrap();
            }
            super::decoder::Conf::Capa(_capa) => {
                let _ = &stream.write(b"+OK\r\n").unwrap();
            }
        },
        Psync => {
            let _ = &stream
                .write(b"+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n")
                .unwrap();

            let empty = BASE64_STANDARD.decode(crate::EMPTY_RDB).unwrap();

            println!("{}", String::from_utf8_lossy(&empty));

            let mut data = format!("${}\r\n", empty.len()).as_bytes().to_vec();

            data.extend(&empty);

            let _ = &stream.write(&data).unwrap();
        }
        OkStatus => (),
    };
}
