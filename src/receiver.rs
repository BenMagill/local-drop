mod message;
mod peer;
use clap::Parser;
use indicatif::ProgressBar;
use message::{Message, Stream};
use peer::PeerService;
use requestty::{prompt_one, Question};
use std::fs;
use std::net::TcpListener;
use std::path::Path;

#[derive(Parser, Debug)]
struct Args {
    // Name to show to people wanting to send
    #[arg(short, long, default_value_t = String::from("local_drop"))]
    name: String,

    // Port to run on
    #[arg(short, long, default_value_t = 12727)]
    port: u16,

    #[arg(short, long, default_value_t = String::from("./data/"))]
    data_dir: String,
}

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

fn main() {
    let args = Args::parse();

    // Create a service
    let port = args.port;
    let tcp_listener = TcpListener::bind(String::from("0.0.0.0:") + &port.to_string()).unwrap();

    PeerService::announce(&args.name, port);

    // intentionally not moving stream to thread so that only one request processed at a time
    for stream in tcp_listener.incoming() {
        let stream = stream.unwrap();
        let mut s = Stream::new(stream);

        let buf = s.read();

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
                            let file_recv_buf = s.read_amount_closure(length_left, |bytes_done| {
                                pb.set_position(bytes_done as u64);
                            });

                            // TODO: check for missmatch between file_recv_buf and info.file_size

                            pb.finish_and_clear();

                            // TODO: make this path customisable
                            let path = Path::new(&args.data_dir).join(info.file_name);
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
