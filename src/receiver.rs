use local_drop::read_stream;
use std::any::Any;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
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

    thread::spawn(|| {
        let mut service = MdnsService::new(
            ServiceType::new("localdrop", "tcp").unwrap(),
            port.parse().unwrap(),
        );

        let mut txt_record = TxtRecord::new();
        let context: Arc<Mutex<Context>> = Arc::default();

        txt_record.insert("name", "raspberrrrrrry").unwrap();

        service.set_registered_callback(Box::new(on_service_reg));
        service.set_context(Box::new(context));
        service.set_txt_record(txt_record);

        let event_loop = service.register().unwrap();
        loop {
            event_loop.poll(Duration::from_secs(5)).unwrap();
        }
    });
    for stream in tcp_listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Got connection");
        let buf = read_stream(&mut stream);
        dbg!(buf);
        //let mut buf = [0; 1028];
        //stream.read(&mut buf).unwrap();
        //dbg!(buf);
        stream.write("ok".as_bytes()).unwrap();
    }
}

fn on_service_reg(result: zeroconf::Result<ServiceRegistration>, context: Option<Arc<dyn Any>>) {
    let service = result.unwrap();
    println!("Registered: {:?}", service);

    let context = context
        .as_ref()
        .unwrap()
        .downcast_ref::<Arc<Mutex<Context>>>()
        .unwrap()
        .clone();

    context.lock().unwrap().service_name = service.name().clone();

    println!("Context: {:?}", context);
}
//fn main() {
//// setup some sort of server
//// listen for hello messages
//// ask user if want to recieve file from this person
////      maybe get info about size and name as well?
//// send ok to the sender with a unique code
//// recieve data from sender
//}
