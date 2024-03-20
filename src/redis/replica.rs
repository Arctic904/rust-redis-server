use std::{borrow::BorrowMut, io::Write, net::TcpStream};

use crate::Replica;

pub fn connect_to_master(replica: Replica, port: u16) {
    //comment
    let connection = TcpStream::connect(format!("{}:{}", replica.host, replica.port));
    let mut binding = connection.expect("Misconfigured Master, invalid port or host");
    let stream = binding.borrow_mut();

    let _ = &stream.write(b"*1\r\n$4\r\nping\r\n").unwrap();

    let _ = &stream
        .write(
            format!(
                "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
                port.to_string().len(),
                port
            )
            .as_bytes(),
        )
        .unwrap();
    let _ = &stream
        .write(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n")
        .unwrap();

    let _ = &stream
        .write(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .unwrap();
}
