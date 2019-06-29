use pnet::datalink::MacAddr;
//use reqwest::{Client, Response};
use std::mem;
use std::io::{self, Cursor};
use futures::{stream, Future, Stream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use reqwest::r#async::Client;
// 0.9.14
use tokio;
use reqwest::r#async::Chunk;

pub fn lookup_mac(addresses: Vec<MacAddr>) -> Result<HashMap<MacAddr, String>, Box<Error>> {
    let client = Client::new();
    let mut result: HashMap<MacAddr, String> = HashMap::new();

    let urls: Vec<(String, &MacAddr)> = addresses.iter().map(|address| (format!("https://api.macvendors.com/{}", address), address)).collect();
    for (url, address) in urls {
        let body = match reqwest::get(url.as_str()) {
            Ok(mut r) => {
                match r.text() {
                    Ok(body) => Some(body),
                    Err(_) => None
                }
            },
            Err(_) => None
        };
        if body.is_some() {
            result.insert(address.clone(), body.unwrap());
        }
        thread::sleep(Duration::from_secs(1));
    }
    Ok(result)
}