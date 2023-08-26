use std::{io::Read, net::TcpStream};

pub enum MessageType {
    Ask = 0b0000_0001,
    AskOk = 0b0000_0010,
    Data = 0b0000_0011,
    DataEnd = 0b0000_0100,
}

pub struct Message {}

impl Message {
    pub fn parse(message: Vec<u8>) -> Result<MessageType, ()> {
        if message.len() == 0 {
            return Err(());
        }

        match message.get(0).unwrap() {
            1 => Ok(MessageType::Ask),
            2 => Ok(MessageType::AskOk),
            3 => Ok(MessageType::Data),
            4 => Ok(MessageType::DataEnd),
            _ => Err(()),
        }
    }

    /**
     * byte 1 is always for the message type
     * bytes 2 - 5 is the file_size
     * bytes 6 is for the length of the file_name (up to 254 chars)
     */
    pub fn build_ask(file_name: &str, file_size: u32) -> [u8; 300] {
        // TODO: currently will send file_name in a fixed number of bytes
        // should change to allow for variable length information to be sent
        let mut bytes = [0; 300];
        bytes[0] = MessageType::Ask as u8;
        let fs_bytes = file_size.to_be_bytes();
        dbg!(file_size);
        bytes[1] = fs_bytes[0];
        bytes[2] = fs_bytes[1];
        bytes[3] = fs_bytes[2];
        bytes[4] = fs_bytes[3];

        let mut fn_bytes = file_name.as_bytes();
        if fn_bytes.len() > 255 {
            fn_bytes = &fn_bytes[0..255];
        }
        bytes[5] = fn_bytes.len() as u8;

        bytes[6..6 + fn_bytes.len()].copy_from_slice(fn_bytes);

        bytes
    }

    pub fn build_ask_ok() {}

    pub fn build_data_packet(data: [u8; 1027]) {}

    pub fn build_data_end() {}
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
