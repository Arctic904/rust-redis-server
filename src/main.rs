mod redis;

// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::BufReader,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use clap::Parser;

use crate::redis::handle_input::read_input;

#[derive(Debug, Clone)]
pub struct Replica {
    host: String,
    port: u16,
}

const NULL_REPLY: &[u8; 5] = b"$-1\r\n";
const OK_REPLY: &[u8; 5] = b"+OK\r\n";

#[derive(Debug, Parser)]
#[clap(name = "redis-server", version)]
pub struct RedisServer {
    #[clap(long, default_value_t = 6379)]
    port: u16,

    #[clap(
        long,
        value_delimiter = ' ',
        num_args = 2,
        value_names = vec!["HOST", "PORT"],
        help = "Makes this server a replica of <HOST>:<PORT>",
    )]
    replicaof: Option<Vec<String>>,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // RedisServer::command().

    let args = RedisServer::parse();

    let replica = args.replicaof.map(|rep| Replica {
        host: rep.first().unwrap().to_string(),
        port: rep.get(1).unwrap().parse().unwrap(),
    });

    println!("Port: {}\n{:?}", args.port, replica);

    let data_store = Arc::new(Mutex::new(HashMap::<String, redis::Data>::new()));

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).unwrap();

    if let Some(replica) = replica.clone() {
        redis::replica::connect_to_master(replica, args.port);
    }

    for stream in listener.incoming() {
        let replica = replica.clone();
        let cloned_store = Arc::clone(&data_store);
        let _test = thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut bufreader = BufReader::new(stream.try_clone().unwrap());
                read_input(&mut bufreader, &mut stream, cloned_store, replica);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}
