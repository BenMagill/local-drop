mod message;
mod peer;
use clap::Parser;
use message::{Message, Stream};
use requestty::Question;
use std::net::{IpAddr, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{env, fs::File, io::Read, thread};

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

#[derive(Parser, Debug)]
struct Args {
    // File to send
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    let p = PeerService::new();
    p.start_search();

    // provide the filename to send when running for simplicity
    //let args: Vec<String> = env::args().collect();
    let file_path = &args.file;
    let mut buffer: Vec<u8> = Vec::new();
    get_file(&mut buffer, file_path);
    let file_name = Path::new(file_path).file_name().unwrap();
    let file_size = buffer.len() * 8;

    // search for devices running it
    println!("Finding devices...");
    thread::sleep(Duration::from_secs(3));
    p.end_search();

    let s = p.get_peers();
    for (addr, service) in s.iter() {
        let addr: IpAddr = addr.parse().unwrap();

        if addr.is_ipv6() {
            continue;
        }
        println!("{} ({})", service.name, addr);
    }

    let mut choices = Vec::new();
    let mut service_addr = Vec::new();

    for p in s.values().cloned() {
        if p.address.parse::<IpAddr>().unwrap().is_ipv4() {
            choices.push(format!("{} ({})", p.name, p.address));
            service_addr.push(p.address);
        }
    }

    let q = Question::select("peer")
        .message("Select a device to send to")
        .choices(choices)
        .build();
    let choice = requestty::prompt_one(q);
    let index = choice.unwrap().as_list_item().unwrap().index;
    let addr = service_addr.get(index).unwrap().clone();

    let service = s.get(&addr).unwrap();
    let listener = addr + ":" + service.port.to_string().as_str();

    let stream = TcpStream::connect(listener).unwrap();
    let mut s = Stream::new(stream);

    let ask_msg = Message::build_ask(file_name.to_str().unwrap(), file_size as u32);
    s.write(ask_msg);

    println!("Requesting to send file...");
    let buf = s.read();

    match Message::parse(buf) {
        Ok(Message::AskOk) => {
            println!("Request accepted");
            let buf = Message::build_data(&buffer);
            s.write(buf);

            let buf = s.read();
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
