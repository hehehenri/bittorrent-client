use std::net::Ipv4Addr;

use url::Url;

use crate::{parser::BencodeTrackerResponse, Client, Peer, Torrent};

impl Torrent {
    fn tracker_url(&self, peer_id: &str) -> Url {
        let mut url = self.announce.clone();

        url.query_pairs_mut()
            .append_pair("info_hash", &self.info_hash.to_string())
            .append_pair("peer_id", &peer_id)
            .append_pair("port", peer_id)
            .append_pair("uploaded", "0")
            .append_pair("downloaded", "0")
            .append_pair("compact", "1")
            .append_pair("left", &self.length.to_string());

        url
    }
}

pub struct TrackerResponse {
    interval: u64,
    peers: Vec<Peer>,
}

fn peer_from_bytes(bytes: &[u8]) -> Result<Peer, String> {
    match bytes {
        &[a, b, c, d, e, f] => {
            let ip = Ipv4Addr::new(a, b, c, d);
            let port = ((e as u16) << 8) | f as u16;

            Ok(Peer { ip, port })
        }
        _ => Err("failed to parse bytes into peer".to_string()),
    }
}

impl TryFrom<BencodeTrackerResponse> for TrackerResponse {
    type Error = ResponseError;

    fn try_from(bencode: BencodeTrackerResponse) -> Result<Self, Self::Error> {
        let peers: Result<Vec<Peer>, String> = bencode
            .peers
            // 4 bytes to represent IPv4 and 2 for the port
            .chunks(4 + 2)
            .map(peer_from_bytes)
            .collect();

        let peers = peers.map_err(|err| ResponseError::Decoding(err.to_string()))?;

        let response = TrackerResponse {
            interval: bencode.interval,
            peers,
        };

        Ok(response)
    }
}

pub enum ResponseError {
    Decoding(String),
    Failed(String),
}

impl Client {
    pub async fn track(&self, torrent: &Torrent) -> Result<TrackerResponse, ResponseError> {
        let tracker_url = torrent.tracker_url(&self.peer_id);

        // TODO: properly handle errors
        let resp = reqwest::get(tracker_url.as_str())
            .await
            .map_err(|e| ResponseError::Failed(e.to_string()))?;

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ResponseError::Decoding(e.to_string()))?
            .to_vec();

        let bencode = BencodeTrackerResponse::try_from(bytes.as_slice())
            .map_err(|err| ResponseError::Decoding(err.to_string()))?;

        TrackerResponse::try_from(bencode)
    }
}
