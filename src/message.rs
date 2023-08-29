pub mod comms;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use comms::{Deserialiser, Serialiser};

#[derive(Clone)]
pub enum MessageType {
    Ask = 0b0000_0001,
    AskOk = 0b0000_0010,
    AskDeny = 0b000_0011,
    Data = 0b0000_0100,
    DataRecvd = 0b0000_0101,
}

impl MessageType {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug)]
pub struct AskInfo {
    pub file_size: u32,
    pub file_name: String,
}

pub enum Message {
    Ask(AskInfo),
    AskOk,
    AskDeny,
    Data,
    DataRecvd,
}

impl Message {
    pub fn parse(v: Vec<u8>) -> Result<Message, ()> {
        let mut d = Deserialiser::from_vec(v);
        //if message.len() == 0 {
        //return Err(());
        //}

        match d.read_u8() {
            1 => {
                let ask = Message::parse_ask(d);
                dbg!(&ask);
                Ok(Message::Ask(ask))
            }
            2 => Ok(Message::AskOk),
            3 => Ok(Message::AskDeny),
            4 => Ok(Message::Data),
            5 => Ok(Message::DataRecvd),
            _ => Err(()),
        }
    }

    /**
     * byte 1 is always for the message type
     * bytes 2 - 5 is the file_size
     * bytes 6 is for the length of the file_name (up to 254 chars)
     */
    pub fn build_ask(file_name: &str, file_size: u32) -> Vec<u8> {
        let mut s = Serialiser::new();

        s.add(MessageType::Ask);
        s.add(file_size);
        s.add(file_name);

        //let mut bytes = vec![0; 6];
        //bytes[0] = MessageType::Ask.to_u8();

        //let fs_bytes = file_size.to_be_bytes();
        //bytes[1] = fs_bytes[0];
        //bytes[2] = fs_bytes[1];
        //bytes[3] = fs_bytes[2];
        //bytes[4] = fs_bytes[3];

        //// Only use the first 255 bytes for the file name
        //let mut fn_bytes = file_name.as_bytes();
        //if fn_bytes.len() > 255 {
        //fn_bytes = &fn_bytes[0..255];
        //}
        //bytes[5] = fn_bytes.len() as u8;

        //bytes.extend_from_slice(fn_bytes);

        s.output()
    }

    fn parse_ask(mut d: Deserialiser) -> AskInfo {
        // 1-4 = file size
        // 5 = file name len
        // ... = file name

        let file_size = d.read_u32();
        let file_name = d.read_string();

        AskInfo {
            file_size,
            file_name,
        }
        //let b = message.as_slice();
        //let s_length = usize::from(*message.get(5).unwrap());

        //AskInfo {
        //file_size: u32::from_be_bytes(b[1..5].try_into().unwrap()),
        //file_name: String::from_utf8(b[6..6 + s_length].try_into().unwrap()).unwrap(),
        //}
    }

    pub fn build_ask_ok() -> Vec<u8> {
        let bytes: Vec<u8> = vec![MessageType::AskOk.to_u8()];

        bytes
    }

    pub fn build_ask_deny() -> Vec<u8> {
        let bytes: Vec<u8> = vec![MessageType::AskDeny.to_u8()];

        bytes
    }

    pub fn build_data(data: &Vec<u8>) -> Vec<u8> {
        let mut v = vec![MessageType::Data.to_u8()];
        v.extend_from_slice(data);
        v
    }

    pub fn build_data_received() -> Vec<u8> {
        let bytes: Vec<u8> = vec![MessageType::DataRecvd.to_u8()];

        bytes
    }
}

/**
* Attempt to read x bytes
* If less recieved then just returns what was recieved
*/
pub fn read_bytes(mut stream: &TcpStream, size: usize) -> Vec<u8> {
    let mut buf = vec![0; size];
    let length = stream.read(&mut buf).unwrap();

    buf[0..length].to_vec()
}

pub struct Stream {
    stream: TcpStream,
}
impl Stream {
    pub fn new(stream: TcpStream) -> Stream {
        Stream { stream }
    }

    pub fn read(&mut self) -> Vec<u8> {
        let message_size_b = read_bytes(&self.stream, 4);
        let message_size = u32::from_be_bytes(message_size_b.try_into().unwrap());

        let mut buf = vec![];
        while buf.len() < message_size as usize {
            let bytes = read_bytes(&self.stream, message_size as usize - buf.len());
            buf.extend_from_slice(&bytes);
        }

        buf
    }

    pub fn read_first_byte(&mut self) -> (u32, u8) {
        let message_size_b = read_bytes(&self.stream, 4);
        let message_size = u32::from_be_bytes(message_size_b.try_into().unwrap());
        let byte = read_bytes(&self.stream, 1)[0];

        (message_size - 1, byte)
    }

    pub fn read_amount_closure<F>(&mut self, amount: u32, clsr: F) -> Vec<u8>
    where
        F: Fn(usize) -> (),
    {
        let mut buf = vec![];
        while buf.len() < amount as usize {
            let bytes = read_bytes(&self.stream, amount as usize - buf.len());
            buf.extend_from_slice(&bytes);
            clsr(buf.len());
        }
        buf
    }

    pub fn write(&mut self, bytes: Vec<u8>) {
        let mut output = vec![0; 4];
        let size = bytes.len() as u32;
        let size_bytes = size.to_be_bytes();
        output[0] = size_bytes[0];
        output[1] = size_bytes[1];
        output[2] = size_bytes[2];
        output[3] = size_bytes[3];

        output.extend_from_slice(&bytes);

        self.stream.write(&output).unwrap();
    }
}
