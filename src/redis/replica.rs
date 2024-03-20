use std::{borrow::BorrowMut, io::Write, net::TcpStream};

use crate::Replica;

pub fn connect_to_master(replica: Replica) {
    //comment
    let connection = TcpStream::connect(format!("{}:{}", replica.host, replica.port));
    let stream = connection
        .expect("Misconfigured Master, invalid port or host")
        .borrow_mut();

    &stream.write(b"+PONG\r\n").unwrap();
}
