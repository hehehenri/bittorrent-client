use rand::prelude::*;
use std::net::Ipv4Addr;
use url::Url;

pub mod parser;
pub mod tracker;

const PORT: usize = 6881;

pub struct Client {
    peer_id: String,
}

impl Client {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 20] = rng.gen();
        let peer_id = bytes.into_iter().map(|b| format!("{:02X}", b)).collect();

        Client { peer_id }
    }
}

#[derive(Debug)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

#[derive(Debug)]
pub struct Torrent {
    pub announce: Url,
    pub info_hash: u64,
    pub hashes: Vec<u8>,
    pub pieces_length: u64,
    pub name: String,
    pub length: u64,
}

fn generate_peer_id() -> String {
    let mut rng = rand::thread_rng();

    (0..20)
        .map(|_| rng.gen::<u8>())
        .into_iter()
        .map(|b| format!("{:02X}", b))
        .collect()
}
