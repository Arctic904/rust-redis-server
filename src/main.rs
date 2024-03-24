mod redis;

// Uncomment this block to pass the first stage
use std::{
    cell::RefCell,
    collections::HashMap,
    io::BufReader,
    net::{TcpListener, TcpStream},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use clap::Parser;
use redis::replica;

use crate::redis::handle_input::read_input;

#[derive(Debug, Clone)]
pub struct Replica {
    host: String,
    port: u16,
}

pub type DataStore = Arc<Mutex<HashMap<String, redis::Data>>>;

const NULL_REPLY: &[u8; 5] = b"$-1\r\n";
const OK_REPLY: &[u8; 5] = b"+OK\r\n";
const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

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

#[derive(Debug, Clone)]
pub struct ReplicaConn {
    host: String,
    port: u16,
    stream: Arc<Mutex<TcpStream>>,
    buffer: Arc<Mutex<BufReader<TcpStream>>>,
}

#[derive(Debug, Clone)]
pub struct Context {
    bufreader: Rc<RefCell<BufReader<TcpStream>>>,
    stream: Rc<RefCell<TcpStream>>,
    store: DataStore,
    replica_conn: Option<ReplicaConn>,
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

    // println!("Port: {}\n{:?}", args.port, replica);

    println!("127.0.0.1:{}", args.port);

    let data_store = Arc::new(Mutex::new(HashMap::<String, redis::Data>::new()));

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).unwrap();

    let (rep_stream, rep_buffer) = if let Some(replica) = replica.clone() {
        redis::replica::connect_to_master(replica, args.port)
    } else {
        (None, None)
    };

    let replica_conn = if let Some(replica) = replica {
        Some(ReplicaConn {
            host: replica.host,
            port: replica.port,
            stream: Arc::new(Mutex::new(rep_stream.unwrap())),
            buffer: Arc::new(Mutex::new(rep_buffer.unwrap())),
        })
    } else {
        None
    };

    for stream in listener.incoming() {
        let cloned_store = Arc::clone(&data_store);
        let replica_conn = replica_conn.clone();
        let _test = thread::spawn(move || match stream {
            Ok(stream) => {
                println!(
                    "accepted new connection at {}",
                    stream.peer_addr().expect("Error")
                );
                let bufreader = BufReader::new(stream.try_clone().unwrap());
                read_input(Context {
                    bufreader: Rc::new(RefCell::new(bufreader)),
                    stream: Rc::new(RefCell::new(stream)),
                    store: cloned_store,
                    replica_conn,
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}
