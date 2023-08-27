use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub enum MessageType {
    Ask = 0b0000_0001,
    AskOk = 0b0000_0010,
    AskDeny = 0b000_0011,
    Data = 0b0000_0100,
    //DataEnd = 0b0000_0100,
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
    //DataEnd,
}

impl Message {
    pub fn parse(message: Vec<u8>) -> Result<Message, ()> {
        if message.len() == 0 {
            return Err(());
        }

        match message.get(0).unwrap() {
            1 => {
                let ask = Message::parse_ask(message);
                Ok(Message::Ask(ask))
            }
            2 => Ok(Message::AskOk),
            3 => Ok(Message::AskDeny),
            4 => Ok(Message::Data),
            //4 => Ok(Message::DataEnd),
            _ => Err(()),
        }
    }

    /**
     * byte 1 is always for the message type
     * bytes 2 - 5 is the file_size
     * bytes 6 is for the length of the file_name (up to 254 chars)
     */
    pub fn build_ask(file_name: &str, file_size: u32) -> Vec<u8> {
        let mut bytes = vec![0; 6];
        bytes[0] = MessageType::Ask.to_u8();

        let fs_bytes = file_size.to_be_bytes();
        bytes[1] = fs_bytes[0];
        bytes[2] = fs_bytes[1];
        bytes[3] = fs_bytes[2];
        bytes[4] = fs_bytes[3];

        // Only use the first 255 bytes for the file name
        let mut fn_bytes = file_name.as_bytes();
        if fn_bytes.len() > 255 {
            fn_bytes = &fn_bytes[0..255];
        }
        bytes[5] = fn_bytes.len() as u8;

        bytes.extend_from_slice(fn_bytes);

        bytes
    }

    fn parse_ask(message: Vec<u8>) -> AskInfo {
        // 1-4 = file size
        // 5 = file name len
        // ... = file name

        let b = message.as_slice();
        let s_length = usize::from(*message.get(5).unwrap());

        AskInfo {
            file_size: u32::from_be_bytes(b[1..5].try_into().unwrap()),
            file_name: String::from_utf8(b[6..6 + s_length].try_into().unwrap()).unwrap(),
        }
    }

    pub fn build_ask_ok() -> Vec<u8> {
        let bytes: Vec<u8> = vec![MessageType::AskOk.to_u8()];

        bytes
    }

    pub fn build_ask_deny() -> Vec<u8> {
        let bytes: Vec<u8> = vec![MessageType::AskDeny.to_u8()];

        bytes
    }

    pub fn send_data(mut stream: &TcpStream, data: &Vec<u8>) {
        stream.write(&[MessageType::Data.to_u8()]).unwrap();
        stream.write(data).unwrap();
    }
}

/**
* ask if can send
* recieve ok
* keep sending some data
* until send data end
**/

pub fn read_stream(mut stream: &TcpStream) -> Vec<u8> {
    let mut buf = [0; 1028];
    let length = stream.read(&mut buf).unwrap();

    buf[0..length].to_vec()
}
