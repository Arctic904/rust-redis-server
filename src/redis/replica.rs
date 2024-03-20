use std::{
    borrow::BorrowMut,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

use crate::Replica;

pub fn connect_to_master(
    replica: Replica,
    port: u16,
) -> (Option<TcpStream>, Option<BufReader<TcpStream>>) {
    //comment
    let connection = TcpStream::connect(format!("{}:{}", replica.host, replica.port));
    let mut binding = connection.expect("Misconfigured Master, invalid port or host");
    let stream = binding.borrow_mut();

    let _ = &stream.write(b"*1\r\n$4\r\nping\r\n").expect("Ping error");

    let mut buffer = BufReader::new(stream.try_clone().unwrap());
    let mut data = String::new();
    let _ = buffer.read_line(&mut data);
    println!("Data: {}", data);

    let _ = &stream
        .write(
            format!(
                "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
                port.to_string().len(),
                port
            )
            .as_bytes(),
        )
        .expect("Conf 1 error");

    let mut data = String::new();
    let _ = buffer.read_line(&mut data);
    println!("Data: {}", data);

    let _ = &stream
        .write(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n")
        .expect("Conf 2 error");

    let mut data = String::new();
    let _ = buffer.read_line(&mut data);
    println!("Data: {}", data);

    let _ = &stream
        .write(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .expect("Sync error");

    let mut data = String::new();
    let _ = buffer.read_line(&mut data);
    println!("Data: {}", data);
    (Some(binding), Some(buffer))
}
