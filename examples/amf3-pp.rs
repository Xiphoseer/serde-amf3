use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to pretty-print
    #[clap(value_parser)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let bytes = std::fs::read(&args.path).unwrap();
    let value = serde_amf3::deserialize::<serde_json::Value>(&bytes[..]).unwrap();
    let mut serializer = serde_json::Serializer::pretty(std::io::stdout().lock());
    value.serialize(&mut serializer).unwrap();
    println!();
}
