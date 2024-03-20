mod redis;

// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::BufReader,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
use clap::Parser;

use crate::redis::handle_input::read_input;

const NULL_REPLY: &[u8; 5] = b"$-1\r\n";
const OK_REPLY: &[u8; 5] = b"+OK\r\n";

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

    let data_store = Arc::new(Mutex::new(HashMap::<String, redis::Data>::new()));

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
