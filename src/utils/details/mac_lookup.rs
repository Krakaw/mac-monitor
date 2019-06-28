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
use reqwest::r#async::Client; // 0.9.14
use tokio;
use reqwest::r#async::Chunk;

pub fn lookup_mac(addresses: Vec<MacAddr>) -> Result<HashMap<MacAddr, String>, Box<Error>> {
    let client = Client::new();

    let urls:Vec<String> = addresses.clone().iter().map(|address|format!("https://api.macvendors.com/{}", address)).collect();
    let bodies = stream::iter_ok(urls)
        .map(move |url| {
            thread::sleep(Duration::from_secs(1));
            client
                .get(url.as_str())
                .send()
                .and_then(|res| res.into_body())
        })
        .buffer_unordered(1);
    let work = bodies
        .for_each(|b| {
            println!("Got {} bytes", b);
            Ok(())
        })
        .map_err(|e| panic!("Error while processing: {}", e));

    tokio::run(work);

    Ok(HashMap::new())

// Channel for reqwest replies.
//    let (tx, rx): (Sender<(MacAddr, String)>, Receiver<(MacAddr, String)>) = mpsc::channel();
//    let mut count = 0;
//    for address in addresses {
//        count = count + 1;
//        send_mac_responses(address.clone(), tx.clone());
//        thread::sleep(Duration::from_secs(1));
//    }
//
//    thread::sleep(Duration::from_secs(2));
//    let mut result: HashMap<MacAddr, String> = HashMap::new();
//    loop {
//        match rx.try_recv() {
//            Ok((address, vendor)) => {
//                println!("{} {}", address.clone(), vendor.clone());
//                result.insert(address, vendor);
//            },
//            Err(e) => {
//                println!("Could not receive mac lookup {}", e);
//                break;
//            }
//        }
//    }
//    Ok(result)
}



//fn send_mac_responses(address: MacAddr, tx: Sender<(MacAddr, String)>) {
//    thread::spawn(move || {
//        let client = Client::new();
//
//        let result = client.get(format!("https://api.macvendors.com/{}", address).as_str()).send().unwrap().text().unwrap();
//        println!("{}", result);
//        tx.send((address, result)).unwrap();
//
//    });
//
//}