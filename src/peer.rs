use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use zeroconf::{prelude::*, MdnsBrowser, MdnsService, ServiceType, TxtRecord};

#[derive(Default, Debug)]
pub struct Context {
    service_name: String,
}

pub type Peers = HashMap<String, Peer>;

#[derive(Clone, Debug)]
pub struct Peer {
    pub address: String,
    pub port: u16,
    pub name: String,
}

pub struct PeerService {
    peers: Arc<Mutex<Peers>>,
    stop: Arc<Mutex<bool>>,
}

impl PeerService {
    pub fn new() -> PeerService {
        PeerService {
            peers: Arc::default(),
            stop: Arc::default(),
        }
    }

    fn get_service() -> ServiceType {
        ServiceType::new("localdrop", "tcp").unwrap()
    }

    pub fn get_peers(&self) -> Peers {
        self.peers.lock().unwrap().clone()
    }

    pub fn start_search(&self) -> JoinHandle<()> {
        let peers = self.peers.clone();
        let stop = self.stop.clone();
        thread::spawn(move || {
            let mut browser = MdnsBrowser::new(PeerService::get_service());

            browser.set_service_discovered_callback(Box::new(move |result, _context| {
                let result = result.unwrap();
                let name = result.txt().clone().unwrap().get("name").unwrap();
                let addr = result.address().clone();
                let s = Peer {
                    address: addr.clone(),
                    port: result.port().clone(),
                    name,
                };
                peers.lock().unwrap().insert(addr, s);
            }));

            let event_loop = browser.browse_services().unwrap();

            loop {
                if stop.lock().unwrap().clone() {
                    return;
                }
                event_loop.poll(Duration::from_secs(1)).unwrap();
            }
        })
    }

    pub fn end_search(&self) {
        *(self.stop.lock().unwrap()) = true;
    }

    pub fn announce(name: &'static str, port: u16) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut service = MdnsService::new(PeerService::get_service(), port);

            let mut txt_record = TxtRecord::new();
            let context: Arc<Mutex<Context>> = Arc::default();

            txt_record.insert("name", name).unwrap();

            service.set_registered_callback(Box::new(|result, context| {
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
            }));
            service.set_context(Box::new(context));
            service.set_txt_record(txt_record);

            let event_loop = service.register().unwrap();
            loop {
                event_loop.poll(Duration::from_secs(5)).unwrap();
            }
        })
    }
}
