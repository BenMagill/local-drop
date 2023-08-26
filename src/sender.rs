use crossbeam::atomic::AtomicCell;
use local_drop::{read_stream, Message};
use std::any::Any;
use std::collections::HashMap;
use std::io::{stdin, Write};
use std::net::{IpAddr, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, fs::File, io::Read, sync::mpsc::channel, thread};
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceType};

fn get_file(buffer: &mut Vec<u8>, file_path: &String) {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Could not open file");
        }
    };
    file.read_to_end(buffer).unwrap();
}

type Services = HashMap<String, Service>;

#[derive(Clone, Debug)]
struct Service {
    address: String,
    port: u16,
    name: String,
}

fn start_search(services: Arc<Mutex<Services>>, stop: Arc<Mutex<bool>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut browser = MdnsBrowser::new(ServiceType::new("localdrop", "tcp").unwrap());

        browser.set_service_discovered_callback(Box::new(move |result, _context| {
            let result = result.unwrap();
            let name = result.txt().clone().unwrap().get("name").unwrap();
            //println!("Service discovered: {:?}", result);
            let addr = result.address().clone();
            let s = Service {
                address: addr.clone(),
                port: result.port().clone(),
                name,
            };
            services.lock().unwrap().insert(addr, s);
            //sender.send(s).unwrap();
        }));

        let event_loop = browser.browse_services().unwrap();

        loop {
            if stop.lock().unwrap().clone() {
                return;
            }
            //println!("Checking");
            event_loop.poll(Duration::from_secs(1)).unwrap();
        }
    })
}

fn main() {
    let s: HashMap<String, Service> = HashMap::new();
    let services = Arc::new(Mutex::new(s));
    let stop = Arc::new(Mutex::new(false));
    start_search(services.clone(), stop.clone());

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
    *(stop.lock().unwrap()) = true;
    //search_thread.join().unwrap();
    println!("Please select a device to send to");

    let s = services.lock().unwrap().clone();
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
    dbg!(&ask_msg[0..20]);
    stream.write(&ask_msg).unwrap();

    let buf = read_stream(&mut stream);
    //dbg!(String::from_utf8(buf));
    //let mut buf = [0; 1028];
    //println!("{}", stream.read(&mut buf).unwrap());
    //dbg!(String::from_utf8(buf.to_vec()));

    //loop {
    //dbg!(services.lock().unwrap().clone());
    //}

    // check if can do it
    // wait for response
    // send data
}
