use std::{borrow::BorrowMut, io::Write, net::TcpStream};

use crate::Replica;

pub fn connect_to_master(replica: Replica) {
    //comment
    let connection = TcpStream::connect(format!("{}:{}", replica.host, replica.port));
    let mut binding = connection.expect("Misconfigured Master, invalid port or host");
    let stream = binding.borrow_mut();

    let _ = &stream.write(b"+PING\r\n").unwrap();
}
