mod peer;
use indicatif::ProgressBar;
use local_drop::comms::Deserialiser;
use local_drop::{Message, Stream};
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
        let mut s = Stream::new(stream);

        let buf = s.read();
        //let buf = read_stream(&mut stream);

        // Expect data to be a Ask message
        match Message::parse(buf) {
            Ok(Message::Ask(info)) => {
                println!(
                    "Someone would like to send a file \n\tFile name: {} \n\tSize: {}",
                    info.file_name, info.file_size
                );
                let accept = prompt_one(
                    Question::confirm("receive")
                        .message("Accept?")
                        .default(true)
                        .build(),
                );

                if accept.unwrap().as_bool().unwrap() {
                    println!("Receiving file");

                    s.write(Message::build_ask_ok());

                    // SHould read the 4 length bytes then the first byte
                    let (length_left, byte) = s.read_first_byte();

                    match Message::parse(vec![byte]) {
                        Ok(Message::Data) => {
                            // TODO: this is messy
                            let pb = ProgressBar::new(length_left as u64);
                            let file_recv_buf = s.read_amount_closure(length_left, |n_bytes| {
                                pb.set_position(n_bytes as u64);
                                //file_recv_buf.extend_from_slice(&bytes);
                            });

                            // TODO: check for missmatch between file_recv_buf and info.file_size

                            //while file_recv_buf.len() < info.file_size as usize / 8 {
                            ////println!("reading more of file");
                            //let buf = read_stream(&stream);
                            ////println!("Got {} bytes", buf.len());

                            //}

                            pb.finish_and_clear();

                            // TODO: make this path customisable
                            let path = Path::new("./data/").join(info.file_name);
                            fs::write(&path, file_recv_buf).unwrap();

                            println!("File has been saved to {}", path.to_str().unwrap());

                            s.write(Message::build_data_received());
                        }
                        _ => panic!("unexpected"),
                    };
                } else {
                    s.write(Message::build_ask_deny());
                }
            }
            _ => println!("Error, expected Ask msg"),
        };

        println!("comms end");
    }
}
