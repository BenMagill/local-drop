mod peer;
use local_drop::{read_stream, Message};
use peer::PeerService;
use requestty::{prompt_one, Question};
use std::fs;
use std::io::{stdin, Write};
use std::net::TcpListener;
use std::path::Path;
use zeroconf::prelude::*;

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
                let accept = prompt_one(Question::confirm("receive")
                    .message(
                        format!("Someone would like to send a file \nFile name: {} \nSize: {} \nAccept?",
                            info.file_name, info.file_size)
                    )
                    .build());

                if accept.unwrap().as_bool().unwrap() {
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
