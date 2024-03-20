use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::{NULL_REPLY, OK_REPLY};

use super::decoder::{SetFlag, Ttl};
use super::Data;

pub fn set_parse(
    set: super::decoder::Set,
    stream: &mut TcpStream,
    data_store: Arc<Mutex<HashMap<String, Data>>>,
) {
    let mut store = data_store.lock().unwrap();
    let binding = store.clone();
    let prev = binding.get(&set.key);
    if let Some(flag) = set.flag {
        match flag {
            SetFlag::Nx => {
                if prev.is_some() {
                    return;
                }
            }
            SetFlag::Xx => {
                if prev.is_none() {
                    return;
                }
            }
        }
    }
    if set.get && prev.is_none() {
        let _ = stream.write(NULL_REPLY);
        return;
    }
    let data = Data {
        value: set.value,
        expires_at: match set.ttl {
            Some(ttl) => match ttl {
                Ttl::Keepttl => {
                    if let Some(prev) = prev {
                        prev.expires_at
                    } else {
                        None
                    }
                }
                Ttl::Ex(ttl) => Some(ttl),
                Ttl::Px(ttl) => Some(ttl),
                Ttl::Exat(ttl) => Some(ttl),
                Ttl::Pxat(ttl) => Some(ttl),
            },
            None => None,
        },
    };
    store.insert(set.key, data);
    if set.get {
        let _ = stream.write(
            format!(
                "${}\r\n{}\r\n",
                prev.unwrap().value.len(),
                prev.unwrap().value
            )
            .as_bytes(),
        );
        return;
    }
    let _ = stream.write(OK_REPLY).unwrap();
}
