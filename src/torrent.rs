use std::fmt;
use std::path::PathBuf;

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Hashes(Vec<[u8; 20]>);

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
        let items: Vec<[u8; 20]> = items.map_err(|e| de::Error::custom("cannot parse [u8; 20]"))?;
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

#[derive(Deserialize, Debug)]
pub(crate) struct Info {
    pub length: usize,
    pub name: PathBuf,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
}

#[derive(Deserialize, Debug)]
pub(crate) struct TorrentFile {
    pub announce: String,
    pub info: Info,
}
