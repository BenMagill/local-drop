mod peer;
use local_drop::{read_stream, Message};
use std::io::{stdin, Write};
use std::net::{IpAddr, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{env, fs::File, io::Read, thread};
use zeroconf::prelude::*;

use crate::peer::PeerService;

fn get_file(buffer: &mut Vec<u8>, file_path: &String) {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Could not open file");
        }
    };
    file.read_to_end(buffer).unwrap();
}

fn main() {
    let p = PeerService::new();
    p.start_search();

    // provide the filename to send when running for simplicity
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let mut buffer: Vec<u8> = Vec::new();
    get_file(&mut buffer, file_path);
    let file_name = Path::new(file_path).file_name().unwrap();
    let file_size = buffer.len() * 8;

    // search for devices running it
    println!("Finding devices...");
    thread::sleep(Duration::from_secs(3));
    p.end_search();
    println!("Please select a device to send to");

    let s = p.get_peers();
    for (addr, service) in s.iter() {
        let addr: IpAddr = addr.parse().unwrap();

        if addr.is_ipv6() {
            continue;
        }
        println!("{} ({})", service.name, addr);
    }

    // select the device to send to
    let mut addr = String::new();
    stdin().read_line(&mut addr).unwrap();
    addr = addr.trim_end().to_string();

    let service = s.get(&addr).unwrap();
    let listener = addr + ":" + service.port.to_string().as_str();

    let mut stream = TcpStream::connect(listener).unwrap();

    let ask_msg = Message::build_ask(file_name.to_str().unwrap(), file_size as u32);
    //dbg!(&ask_msg[0..20]);
    stream.write(&ask_msg).unwrap();

    println!("Requesting to send file...");
    let buf = read_stream(&mut stream);

    match Message::parse(buf.to_vec()) {
        Ok(Message::AskOk) => {
            println!("Request accepted");
            Message::send_data(&stream, &buffer);

            let buf = read_stream(&stream);
            match Message::parse(buf) {
                Ok(Message::DataRecvd) => {
                    println!("File was received ok")
                }
                _ => panic!("unexpected"),
            }
        }
        Ok(Message::AskDeny) => {
            println!("denied ")
        }
        _ => {
            panic!("not expected")
        }
    };

    println!("fin");
}
