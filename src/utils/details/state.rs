use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::borrow::Cow;
// Cow = clone on write
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::{error::Error, fmt};
use std::collections::HashMap;
use pnet_datalink::MacAddr;


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StateProperties {
    pub stored_vendors: HashMap<String, String>,
    pub monitor_macs: Vec<String>,
}

pub struct State {
    pub state_path: String,
    pub properties: StateProperties,
}

impl State {
    pub fn load(state_path: String) -> State {
        let data = fs::read_to_string(state_path.clone()).unwrap_or("{}".to_string());
        let properties:StateProperties  = serde_json::from_str(data.as_str()).unwrap_or(StateProperties{..Default::default()});
        let state = State {
            state_path: state_path.to_string(),
            properties,
        };
        state
    }

    pub fn save(&self) -> std::io::Result<()> {
        let contents: String = serde_json::to_string(&self.properties)?;
        let state_path = self.state_path.clone();
        fs::write(state_path.clone(), contents).expect(format!("Unable to write file {}", state_path).as_str());
        Ok(())
    }
}

pub fn read_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath).expect(format!("Unable to read file: {}", filepath).as_str());
    contents
}

