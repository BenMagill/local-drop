use crossbeam::atomic::AtomicCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs::File, io::Read, sync::mpsc::channel, thread};
use zeroconf::prelude::*;
use zeroconf::{MdnsBrowser, ServiceType};

fn get_file(buffer: &mut Vec<u8>) {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Could not open file");
        }
    };
    file.read_to_end(buffer).unwrap();
}

#[derive(Clone, Debug)]
struct Service {
    address: String,
    port: u16,
}

fn start_search(services: Arc<Mutex<Vec<Service>>>) {
    thread::spawn(move || {
        let mut browser = MdnsBrowser::new(ServiceType::new("nonas", "tcp").unwrap());

        browser.set_service_discovered_callback(Box::new(move |result, _context| {
            let result = result.unwrap();
            //println!("Service discovered: {:?}", result);
            let s = Service {
                address: result.address().clone(),
                port: result.port().clone(),
            };
            services.lock().unwrap().push(s);
            //sender.send(s).unwrap();
        }));

        let event_loop = browser.browse_services().unwrap();

        loop {
            println!("Checking");
            event_loop.poll(Duration::from_secs(1)).unwrap();
        }
    });
}

fn main() {
    let s: Vec<Service> = Vec::new();
    let services = Arc::new(Mutex::new(s));
    start_search(services.clone());

    // provide the filename to send when running for simplicity
    let mut buffer: Vec<u8> = Vec::new();
    get_file(&mut buffer);

    // search for devices running it
    // select the device to send to
    // wait for ok
    // send data

    loop {
        dbg!(services.lock().unwrap().clone());
        thread::sleep(Duration::from_secs(3));
    }
}
