mod peer;
use local_drop::{read_stream, Message, MessageType};
use peer::PeerService;
use std::any::Any;
use std::io::{stdin, Read, Write};
use std::net::{IpAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};
use zeroconf::prelude::*;
use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn main() {
    // Create a service
    let port = "12727";
    let tcp_listener = TcpListener::bind(String::from("0.0.0.0:") + port).unwrap();

    PeerService::announce("raspberrrrrrry", port.parse().unwrap());

    // intentionally not moving stream to thread so that only one request processed at a time
    for stream in tcp_listener.incoming() {
        let mut stream = stream.unwrap();
        let buf = read_stream(&mut stream);

        // Expect data to be a Ask message
        match Message::parse(buf) {
            Ok(Message::Ask(info)) => {
                println!(
                    "
                    Someone would like to send a file\n
                    File name: {}\n
                    Size: {}\n
                    Accept? (enter yes)
                    ",
                    info.file_name, info.file_size
                );

                let mut confirm = String::new();
                stdin().read_line(&mut confirm).unwrap();
                confirm = confirm.trim_end().to_uppercase().to_string();

                if String::from("YES") == confirm {
                    println!("accepted,");

                    stream.write(&Message::build_ask_ok()).unwrap();

                    let mut file_recv_buf: Vec<u8> = vec![];

                    let buf = read_stream(&stream);

                    match Message::parse(vec![*buf.get(0).unwrap()]) {
                        Ok(Message::Data) => {
                            file_recv_buf.extend_from_slice(&buf.as_slice()[1..]);
                            while file_recv_buf.len() < info.file_size as usize / 8 {
                                println!("reading more of file");
                                let buf = read_stream(&stream);
                                println!("Got {} bytes", buf.len());
                                file_recv_buf.extend_from_slice(&buf);
                            }

                            println!("finished reading file");

                            fs::write(Path::new("./data/").join(info.file_name), file_recv_buf)
                                .unwrap();

                            stream.write(&Message::build_data_received()).unwrap();
                        }
                        _ => panic!("unexpected"),
                    };
                } else {
                    stream.write(&Message::build_ask_deny()).unwrap();
                }
            }
            _ => println!("Error, expected Ask msg"),
        };

        println!("comms end");
    }
}
