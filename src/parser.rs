extern crate bip_bencode;

use std::hash::{DefaultHasher, Hash, Hasher};

use bip_bencode::{BDecodeOpt, BDictAccess, BRefAccess, BencodeRef};
use url::Url;

use crate::Torrent;

/// Steps:
/// - Parse a torrent file
/// - Connect to the tracker
/// - Get the peers that have the file from the .torrent
/// - Connect to the peers
/// -

#[derive(Hash, Debug)]
pub struct TorrentInfo {
    pieces: Vec<u8>,
    // TODO: check max size
    pieces_length: u64,
    name: String,
    hash: u64,
}

#[derive(Debug)]
pub struct DecodingError(String);

impl ToString for DecodingError {
    fn to_string(&self) -> String {
        self.0
    }
}

fn lookup<'a, 'b>(
    field: &'static str,
    dict: &'a dyn BDictAccess<&[u8], BencodeRef<'b>>,
) -> Result<&'a BencodeRef<'a>, DecodingError> {
    dict.lookup(field.as_bytes()).ok_or_else(|| {
        let message = format!("{} field not found", field);
        DecodingError(message)
    })
}

fn bencode_from_bytes(bytes: &[u8]) -> Result<BencodeRef<'_>, DecodingError> {
    BencodeRef::decode(&bytes, BDecodeOpt::default()).map_err(|err| DecodingError(err.to_string()))
}

fn lookup_string(
    field: &'static str,
    dict: &dyn BDictAccess<&[u8], BencodeRef<'_>>,
) -> Result<String, DecodingError> {
    let value = lookup(field, dict)?;

    value
        .str()
        .ok_or_else(|| {
            let message = format!("failed to parse {} field into string", field);
            DecodingError(message)
        })
        .map(|str| str.to_string())
}

fn lookup_int(
    field: &'static str,
    dict: &dyn BDictAccess<&[u8], BencodeRef<'_>>,
) -> Result<u64, DecodingError> {
    let value = lookup(field, dict)?;

    value
        .int()
        .ok_or_else(|| {
            let message = format!("failed to parse {} field into string", field);
            DecodingError(message)
        })
        .map(|i| i as u64)
}

fn lookup_bytes(
    field: &'static str,
    dict: &dyn BDictAccess<&[u8], BencodeRef<'_>>,
) -> Result<Vec<u8>, DecodingError> {
    let value = lookup(field, dict)?;

    value
        .bytes()
        .ok_or_else(|| {
            let message = format!("failed to parse {} field into string", field);
            DecodingError(message)
        })
        .map(|bytes| bytes.to_vec())
}

impl<'a> TryFrom<&'a BencodeRef<'a>> for TorrentInfo {
    type Error = DecodingError;

    fn try_from(bencode: &'a BencodeRef<'a>) -> Result<Self, Self::Error> {
        let dict = bencode
            .dict()
            .ok_or(DecodingError("failed to parse info into dict".to_string()))?;

        let mut hasher = DefaultHasher::new();
        bencode.hash(&mut hasher);

        let torrent_info = TorrentInfo {
            pieces: lookup_bytes("pieces", dict)?,
            pieces_length: lookup_int("piece length", dict)?,
            name: lookup_string("name", dict)?,
            hash: hasher.finish(),
        };

        Ok(torrent_info)
    }
}

#[derive(Debug)]
pub struct BencodeTorrent {
    announce: Url,
    info: TorrentInfo,
}

impl TryFrom<&[u8]> for BencodeTorrent {
    type Error = DecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bencode = bencode_from_bytes(bytes)?;
        let dict = bencode.dict().ok_or(DecodingError(String::from(
            "failed to access bencode as dict",
        )))?;

        let info = lookup("info", dict)?;

        let announce = lookup_string("announce", dict)?;
        let announce = Url::parse(&announce).map_err(|e| DecodingError(e.to_string()))?;

        let torrent_file = BencodeTorrent {
            announce,
            info: TorrentInfo::try_from(info)?,
        };

        Ok(torrent_file)
    }
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodingError> {
        let bencode_torrent = BencodeTorrent::try_from(bytes)?;
        let length = todo!();

        let torrent = Torrent {
            announce: bencode_torrent.announce,
            info_hash: bencode_torrent.info.hash,
            hashes: bencode_torrent.info.pieces,
            pieces_length: bencode_torrent.info.pieces_length,
            name: bencode_torrent.info.name,
            length,
        };

        Ok(torrent)
    }
}

pub struct BencodeTrackerResponse {
    pub interval: u64,
    pub peers: Vec<u8>,
}

impl TryFrom<&[u8]> for BencodeTrackerResponse {
    type Error = DecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bencode = bencode_from_bytes(bytes)?;
        let dict = bencode.dict().ok_or(DecodingError(String::from(
            "failed to access bencode as dict",
        )))?;

        let response = BencodeTrackerResponse {
            interval: lookup_int("interval", dict)?,
            peers: lookup_bytes("peers", dict)?,
        };

        Ok(response)
    }
}
