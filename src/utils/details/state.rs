use std::fs;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use pnet_datalink::MacAddr;


//
//
//#[derive(Serialize, Deserialize, Debug)]
//struct MacAddrString {
//    #[serde(with = "serialize_mac_address")]
//    data: MacAddr
//}
//
//impl From<MacAddrString> for MacAddr {
//    fn from(mac: MacAddrString) -> Self {
//        mac.data
//    }
//}
//
//impl From<MacAddr> for MacAddrString {
//    fn from(mac: MacAddr) -> Self {
//        MacAddrString {
//            data: mac
//        }
//    }
//}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StateProperties {
    pub stored_vendors: HashMap<MacAddr, String>,
    pub monitor_macs: Vec<MacAddr>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

    pub fn save(&self, state_properties: StateProperties) -> std::io::Result<()> {
        let contents: String = serde_json::to_string(&state_properties)?;
        let state_path = self.state_path.clone();
        fs::write(state_path.clone(), contents).expect(format!("Unable to write file {}", state_path).as_str());
        Ok(())
    }
}