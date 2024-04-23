use std::fs::File;
use std::io::Write;
use std::{io::Read, path::PathBuf};

use clap::{Parser, Subcommand};
use serde_json::{Map, Value};
use sha1::{Digest, Sha1};

mod torrent;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode a bencode object
    Decode {
        /// Encoded object value
        object: String,
    },
    /// Info about a metainfo file
    Info {
        /// Path to a .torrent file
        path: PathBuf,
    },
}

fn parse_bencode_value(value: &str) -> anyhow::Result<(Value, &str)> {
    match value.chars().next() {
        Some('0'..='9') => {
            let (length, content) = value
                .split_once(':')
                .ok_or(anyhow::anyhow!("cannot find :"))?;
            let length = length.parse::<usize>()?;
            if content.len() < length {
                anyhow::bail!(
                    "encoded string is short: {}, expected: {length}",
                    content.len()
                );
            }
            Ok((
                Value::String(content[..length].to_string()),
                &content[length..],
            ))
        }
        Some('i') => {
            let Some(pos) = value.chars().position(|c| c == 'e') else {
                anyhow::bail!("missing 'e' character");
            };

            let number = value[1..pos].parse::<i64>()?;
            Ok((Value::Number(number.into()), &value[pos + 1..]))
        }
        Some('l') => {
            let mut list = Vec::new();
            let mut value = &value[1..];
            loop {
                match value.chars().next() {
                    Some('e') => {
                        value = &value[1..];
                        break;
                    }
                    Some(_) => {}
                    None => {
                        break;
                    }
                }

                let item = parse_bencode_value(value)?;
                list.push(item.0);
                value = item.1;
            }

            Ok((Value::Array(list), value))
        }
        Some('d') => {
            let mut dictionary = Map::new();
            let mut value = &value[1..];
            loop {
                match value.chars().next() {
                    Some('e') => {
                        value = &value[1..];
                        break;
                    }
                    Some(_) => {}
                    None => {
                        break;
                    }
                }

                let (key, next_value) = parse_bencode_value(value)?;
                let (v, next_value) = parse_bencode_value(next_value)?;
                let Value::String(key) = key else {
                    anyhow::bail!("Unexpected map key: {v}");
                };
                value = next_value;

                dictionary.insert(key, v);
            }

            Ok((Value::Object(dictionary), value))
        }
        Some(_) => todo!(),
        None => Ok((Value::Null, "")),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Decode { object } => {
            let value = parse_bencode_value(&object);
            match value {
                Ok(value) => println!("{}", value.0),
                Err(err) => eprintln!("decode failed: {err}"),
            }
        }
        Commands::Info { path } => {
            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let torrent = serde_bencode::de::from_bytes::<torrent::TorrentFile>(&buffer).unwrap();
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);

            let data = serde_bencode::ser::to_bytes(&torrent.info).unwrap();
            let mut hasher = Sha1::new();
            hasher.update(data);
            let hash = hex::encode(hasher.finalize()).to_string();
            println!("Info Hash: {hash}");
        }
    }
}
