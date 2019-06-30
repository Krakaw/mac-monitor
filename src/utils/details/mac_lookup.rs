use pnet::datalink::MacAddr;
use std::thread;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

pub fn lookup_mac(addresses: Vec<MacAddr>, stored_vendors: &HashMap<MacAddr, String>) -> Result<HashMap<MacAddr, String>, Box<Error>> {
    let mut result: HashMap<MacAddr, String> = HashMap::new();

    let urls: Vec<(String, &MacAddr)> = addresses.iter().map(|address| (format!("https://api.macvendors.com/{}", address), address)).collect();
    for (url, address) in urls {
        if stored_vendors.contains_key(address) {
            result.insert(address.clone(), stored_vendors.get(address).unwrap().clone());
        } else {
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

    }
    Ok(result)
}