use pnet::datalink::{MacAddr };
use reqwest::r#async::{Client, Response,Decoder};
use std::mem;
use std::io::{self, Cursor};
use futures::{Future, Stream};

pub struct MacLookup {
    pub address: MacAddr
}

impl MacLookup {
    pub fn vendor(&self) -> impl Future<Item=(), Error=()>{
        let client = Client::new();

        client.get(format!("https://api.macvendors.com/{}", self.address).as_str())
            .send()
            .and_then(|mut res: Response| {
                let body = mem::replace(res.body_mut(), Decoder::empty());
                body.concat2()
//                println!("Mac lookup: {} - {}", self.address, res.body());
            })
            .map_err(|err| println!("request error: {}", err))
            .map(|body| {
                let mut body = Cursor::new(body);
                let _ = io::copy(&mut body, &mut io::stdout())
                    .map_err(|err| {
                        println!("stdout error: {}", err);
                    });
            })
    }
}