extern crate bip_bencode;

use std::hash::{DefaultHasher, Hash, Hasher};

use bip_bencode::{BDecodeOpt, BDictAccess, BRefAccess, BencodeRef};

/// Steps:
/// - Parse a torrent file
/// - Connect to the tracker
/// - Get the peers that have the file from the .torrent
/// - Connect to the peers
/// -

#[derive(Hash, Debug)]
pub struct TorrentInfo {
    pieces: String,
    // TODO: check max size
    pieces_length: u64,
    // TODO: check max size
    length: u64,
    name: String,
    hash: u64,
}

#[derive(Debug)]
pub struct DecodingError(String);

fn lookup<'a, 'b>(
    field: &'static str,
    dict: &'a dyn BDictAccess<&[u8], BencodeRef<'b>>,
) -> Result<&'a BencodeRef<'a>, DecodingError> {
    dict.lookup(field.as_bytes()).ok_or_else(|| {
        let message = format!("{} field not found", field);
        DecodingError(message)
    })
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

impl<'a> TryFrom<&'a BencodeRef<'a>> for TorrentInfo {
    type Error = DecodingError;

    fn try_from(bencode: &'a BencodeRef<'a>) -> Result<Self, Self::Error> {
        let dict = bencode
            .dict()
            .ok_or(DecodingError("failed to parse info into dict".to_string()))?;

        let mut hasher = DefaultHasher::new();
        bencode.hash(&mut hasher);

        let torrent_info = TorrentInfo {
            pieces: lookup_string("pieces", dict)?,
            pieces_length: lookup_int("pieces_length", dict)?,
            length: lookup_int("length", dict)?,
            name: lookup_string("name", dict)?,
            hash: hasher.finish(),
        };

        Ok(torrent_info)
    }
}

#[derive(Debug)]
pub struct TorrentFile {
    announce: String,
    info: TorrentInfo,
}

impl TryFrom<&[u8]> for TorrentFile {
    type Error = DecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bencode = BencodeRef::decode(bytes, BDecodeOpt::default())
            .map_err(|err| DecodingError(err.to_string()))?;
        let dict = bencode.dict().ok_or(DecodingError(String::from(
            "failed to access bencode as dict",
        )))?;

        let info = lookup("info", dict)?;

        let torrent_file = TorrentFile {
            announce: lookup_string("announce", dict)?,
            info: TorrentInfo::try_from(info)?,
        };

        Ok(torrent_file)
    }
}

#[derive(Debug)]
pub struct Torrent {
    announce: String,
    info_hash: u64,
    hashes: String,
    pieces_length: u64,
    length: u64,
    name: String,
}

impl Torrent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodingError> {
        let torrent = TorrentFile::try_from(bytes)?;

        let torrent = Torrent {
            announce: torrent.announce,
            info_hash: torrent.info.hash,
            hashes: torrent.info.pieces,
            pieces_length: torrent.info.pieces_length,
            length: torrent.info.length,
            name: torrent.info.name,
        };

        Ok(torrent)
    }
}
