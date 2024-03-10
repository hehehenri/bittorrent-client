extern crate bip_bencode;

use bittorrent_client::Torrent;

fn main() {
    let bytes = std::fs::read("example.torrent").unwrap();

    let torrent = Torrent::from_bytes(&bytes).unwrap();

    dbg!(torrent.announce);
}
