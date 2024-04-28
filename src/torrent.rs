use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug)]
pub(crate) struct Hashes(pub Vec<[u8; 20]>);

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String array where each piece has 20 items")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let items: Result<Vec<_>, _> = value.chunks(20).map(|c| c.try_into()).collect();
        let items: Vec<[u8; 20]> =
            items.map_err(|e| de::Error::custom(format!("cannot parse [u8; 20]: {e}")))?;
        if let Some(last) = items.last() {
            if last.len() != 20 {
                return Err(de::Error::custom(format!(
                    "wrong length of the last piece: {}",
                    last.len()
                )));
            }
        }
        Ok(Hashes(items))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Hashes, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashVisitor)
    }
}

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data: Vec<u8> = self.0.concat().into_iter().collect();
        serializer.serialize_bytes(&data)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Info {
    pub length: usize,
    pub name: PathBuf,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentFile {
    pub announce: String,
    pub info: Info,
}

#[derive(Debug)]
pub struct Peers(pub Vec<SocketAddrV4>);

#[derive(Deserialize, Debug)]
pub struct TrackerResponse {
    pub _interval: usize,
    pub peers: Peers,
}

struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("array with 6 bytes for each item (4 bytes for addr, 2 bytes for port)")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let items: Vec<_> = value
            .chunks(6)
            .map(|c| {
                SocketAddrV4::new(
                    Ipv4Addr::new(c[0], c[1], c[2], c[3]),
                    (c[4] as u16) << 8 | (c[5] as u16),
                )
            })
            .collect();
        Ok(Peers(items))
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Peers, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}
