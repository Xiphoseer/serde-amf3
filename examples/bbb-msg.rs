use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to pretty-print
    #[clap(value_parser)]
    path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExecutionStateStrip {
    #[serde(rename = "actionIndex")]
    action_index: u32,
    id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExecutionState {
    #[serde(rename = "stateID")]
    state_id: u32,
    strips: Vec<ExecutionStateStrip>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "Type")]
enum Action {
    OnInteract {
        #[serde(rename = "__callbackID__")]
        callback_id: String,
    },
    FlyUp {
        #[serde(rename = "Distance")]
        distance: f64,
        #[serde(rename = "__callbackID__")]
        callback_id: String,
    },
    FlyDown {
        #[serde(rename = "Distance")]
        distance: f64,
        #[serde(rename = "__callbackID__")]
        callback_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Pos2 {
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Strip {
    id: u32,
    actions: Vec<Action>,
    ui: Pos2,
}

#[derive(Debug, Deserialize, Serialize)]
struct State {
    id: u32,
    strips: Vec<Strip>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Root {
    #[serde(rename = "BehaviorID")]
    behavior_id: String,
    #[serde(rename = "executionState")]
    execution_state: ExecutionState,
    #[serde(rename = "objectID")]
    object_id: String,
    states: Vec<State>,
}

fn main() {
    let args = Args::parse();

    let bytes = std::fs::read(&args.path).unwrap();
    let value = serde_amf3::deserialize::<Root>(&bytes[..]).unwrap();
    println!("{:#?}", value);
}
