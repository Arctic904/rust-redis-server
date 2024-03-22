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
    // println!("Data: {}", data);

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
    // println!("Data: {}", data);

    let _ = &stream
        .write(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n")
        .expect("Conf 2 error");

    let mut data = String::new();
    let _ = buffer.read_line(&mut data);
    // println!("Data: {}", data);

    let _ = &stream
        .write(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .expect("Sync error");

    let mut data = String::new();
    let _ = buffer.read_line(&mut data);

    let mut byte_total = String::new();
    let _ = buffer.read_line(&mut byte_total);
    println!("Bytes: {}", byte_total);

    let byte: usize = byte_total
        .split('$')
        .find(|x| !x.is_empty())
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    println!("{:?}", byte);

    let mut data_buf = vec![0; byte];

    let _ = buffer.read_exact(&mut data_buf);

    println!("End: {}", String::from_utf8_lossy(&data_buf));

    (Some(binding), Some(buffer))
}
