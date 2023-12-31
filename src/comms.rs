use std::{io::Read, net::TcpStream};

use crate::message::MessageType;

/**
* Encode and decode communications received
* Given data, it can encode it to binary
* And then on the receiving end, decode it out
*   For decode there should possibly be a way to request more data from a buffer (for stuff like
*   tcp streams) as may not always get all the data at once
*
* Encode
*   a.add_u32(...);
*   a.add_string(...);
*
* Decode
*   a.parse_u32(...);
*   a.parse_string(...);
*/

pub trait Encoder {
    fn to_bytes(&self) -> Vec<u8>;
}

impl Encoder for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl Encoder for &str {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![];

        let mut fn_bytes = self.as_bytes();
        if fn_bytes.len() > 255 {
            fn_bytes = &fn_bytes[0..255];
        }
        v.push(fn_bytes.len() as u8);
        v.extend_from_slice(fn_bytes);
        v
    }
}

impl Encoder for MessageType {
    fn to_bytes(&self) -> Vec<u8> {
        vec![self.clone().to_u8()]
    }
}

pub struct Serialiser {
    output: Vec<u8>,
}

impl Serialiser {
    pub fn new() -> Serialiser {
        Serialiser { output: vec![] }
    }

    pub fn add_u8(&mut self, data: u8) {
        self.output.push(data);
    }

    pub fn add(&mut self, data: impl Encoder) {
        let vec = data.to_bytes();
        self.output.extend(vec);
    }

    pub fn output(&self) -> Vec<u8> {
        self.output.clone()
    }
}

pub struct Deserialiser {
    buf: Vec<u8>,
}

impl Deserialiser {
    pub fn from_vec(buf: Vec<u8>) -> Deserialiser {
        Deserialiser { buf: buf.clone() }
    }

    fn read_bytes(&mut self, amount: usize) -> Vec<u8> {
        // TODO: i think this is inefficient
        let bytes = self.buf[0..amount].to_vec();
        self.buf = self.buf[amount..].to_vec();

        bytes
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes = self.read_bytes(4);
        u32::from_be_bytes(bytes[0..4].try_into().unwrap())
    }

    pub fn read_u8(&mut self) -> u8 {
        let bytes = self.read_bytes(1);
        u8::from_be_bytes(bytes.try_into().unwrap())
    }

    pub fn read_string(&mut self) -> String {
        let string_b = self.read_vec();

        String::from_utf8(string_b.try_into().unwrap()).unwrap()
    }

    pub fn read_vec(&mut self) -> Vec<u8> {
        let s_length = usize::from(*self.read_bytes(1).get(0).unwrap());

        let bytes = self.read_bytes(s_length);

        bytes
    }
}
