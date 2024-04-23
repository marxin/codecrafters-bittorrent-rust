use clap::{Parser, Subcommand};
use serde_json::{Value};

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
    }
}