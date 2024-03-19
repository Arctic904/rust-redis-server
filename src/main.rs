// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread,
};

use bytes::buf;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            match stream {
                Ok(mut stream) => {
                    println!("accepted new connection");
                    let bufreader = BufReader::new(stream.try_clone().unwrap());
                    bufreader.lines().for_each(|line| {
                        let binding = line.unwrap_or("".to_owned());
                        let line = binding.as_str();
                        println!("{}", line);
                        match line {
                            "ping" => {
                                let _ = &stream.write(b"+PONG\r\n").unwrap();
                            }
                            _ => {
                                println!("err: {}", line);
                            } // &stream.write(b"Unknown Command"),
                        };
                    })
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        });
    }
}
