use pnet::datalink::{self, NetworkInterface};

pub struct Interfaces {
    pub interfaces: Vec<NetworkInterface>,
}
impl Interfaces {
    pub fn new() -> Interfaces {
        let interfaces = datalink::interfaces();
        Interfaces {
            interfaces
        }
    }

    pub fn print_list(&self) {
        println!("Listing interfaces:\n");
        for interface in self.interfaces.iter() {
            println!("{}\n", interface);
        }
    }
}