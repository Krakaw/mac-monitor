use clap::{App as ClApp, Arg, ArgMatches};
use pnet::datalink::MacAddr;
use pnet_datalink::NetworkInterface;
use crate::utils;
use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::str::FromStr;
use prettytable::format;
use prettytable::Table;
use crate::utils::details::state::State;


pub struct AppProperties {
    pub list_interfaces: bool,
    pub interface: Option<String>,
    pub poll_in_seconds: usize,
    pub default_debounce: usize,
    pub notify_on_new: bool,
    pub state_file: Option<String>,
    pub monitor_macs: Vec<MacAddr>,
    pub lookup_macs: bool,

}

pub struct App {
    pub interfaces: utils::network::interfaces::Interfaces,
    pub interface: Option<NetworkInterface>,
    pub properties: AppProperties,
    pub mac_addresses: HashMap<MacAddr, Ipv4Addr>,
    pub state: Option<State>,
}

const BANNER: &str = "Arp Notify";
const DEFAULT_POLL: &str = "10";
const DEFAULT_DEBOUNCE: &str = "7";

impl App {
    pub fn new() -> App {
        let app = App::build();
        app
    }

    pub fn build() -> App {
        let matches = get_matches();
        let interfaces = utils::network::interfaces::Interfaces::new();

        let state_file = matches.value_of("state_file").map(|x| x.clone().to_owned());
        let mut state = None;
        if state_file.is_some() {
            state = Some(State::load(state_file.clone().unwrap()));
        }


        let list_interfaces = matches.is_present("list_interfaces");
        let lookup_macs = matches.is_present("lookup_macs");

        let interface = matches.value_of("interface").map(|x| x.clone().to_owned());
        // How often should we poll the network
        let poll_in_seconds: usize = usize::from_str(matches.value_of("poll_time").unwrap_or(DEFAULT_POLL)).unwrap_or(usize::from_str(DEFAULT_POLL).unwrap());
        //After how many polls should a device be considered available
        let default_debounce: usize = usize::from_str(matches.value_of("debounce").unwrap_or(DEFAULT_DEBOUNCE)).unwrap_or(usize::from_str(DEFAULT_DEBOUNCE).unwrap());
        let notify_on_new: bool = matches.value_of("notify_on_new").map(|x| x == "1").unwrap_or(false);



        let mut monitor_macs: Vec<MacAddr> = vec![];
        if matches.is_present("monitor_macs") {
            monitor_macs = matches
                .values_of("monitor_macs")
                .unwrap()
                .map(|x| MacAddr::from_str(x.clone()).unwrap())
                .collect();
        }

        let properties = AppProperties {
            list_interfaces,
            interface,
            poll_in_seconds,
            default_debounce,
            notify_on_new,
            state_file,
            monitor_macs,
            lookup_macs,
        };

        App {
            interfaces,
            interface: None,
            properties,
            mac_addresses: HashMap::new(),
            state
        }
    }

    pub fn process(&mut self) {
        if self.properties.list_interfaces {
            self.interfaces.print_list();
            std::process::exit(0);
        }

        self.set_interface();
        self.fetch_mac_addresses();

        // Fetch the mac lookup

        self.print_macs();
        match &self.state {
            Some(state) => state.save(),
            None => Ok(())
        };
    }

    pub fn print_macs(&self) {
        let mut vendors: HashMap<MacAddr, String> = HashMap::new();
        if self.properties.lookup_macs {
            let macs = self.mac_addresses.keys().map(|&x| x.clone()).collect();
            vendors = utils::details::mac_lookup::lookup_mac(macs).unwrap_or(HashMap::new());
        }

        if self.properties.monitor_macs.len() > 0 {
            for monitor_mac in self.properties.monitor_macs.to_owned() {
                if self.mac_addresses.contains_key(&monitor_mac) {
                    println!("Mac Exists {} - {:?}", &monitor_mac, self.mac_addresses.get(&monitor_mac));
                } else {
                    println!("Mac Un-available {}", &monitor_mac);
                }
            }
        } else {
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            let titles = {
                if self.properties.lookup_macs {
                    row!["host", "mac", "vendor"]
                } else {
                    row!["host", "mac"]
                }
            };
            table.set_titles(titles);


            let mut ips: Vec<(&MacAddr, &Ipv4Addr)> = self.mac_addresses.iter().collect();
            ips.sort_by(|a, b| a.1.cmp(b.1));

            println!("{} devices found\n", &ips.len());
            for (mac, ip) in ips {
                let row = {
                    if self.properties.lookup_macs {
                        row![ip, mac, vendors.get(mac).unwrap_or(&"Unknown".to_string())]
                    } else {
                        row![ip, mac]
                    }
                };
                table.add_row(row);
            }

            if table.len() > 0 {
                table.printstd();
            } else {
                println!("No hosts found...");
            }
        }
    }

    pub fn set_interface(&mut self) {
        let chosen_interface = self.properties.interface.clone().unwrap_or("".to_string());
        let interface_match = |iface: &NetworkInterface| {
            return &iface.name == &chosen_interface;
        };

        let interface = self.interfaces.interfaces.clone()
            .into_iter()
            .filter(interface_match)
            .next()
            .unwrap();

        if interface.is_loopback() {
            println!("Aborting because chosen interface is a loopback interface.\nChoose a non-loopback interface.\n");
            std::process::exit(1);
        }

        if interface.ips.is_empty() {
            println!("Aborting because chosen interface doesn't have a network address.\n");
            std::process::exit(1);
        }

        println!("Using interface: {} - {:?}\n", interface.name, interface.ips.iter().filter_map(|x: &IpNetwork| {
            match x.ip() {
                IpAddr::V4(value) => Some(value.to_string()),
                _ => None
            }
        }).collect::<Vec<String>>());

        self.interface = Some(interface);
    }

    pub fn fetch_mac_addresses(&mut self) -> HashMap<MacAddr, Ipv4Addr> {
        let arp_requests = utils::network::arp::Arp {
            interface: self.interface.clone().unwrap()
        };
        let available_macs = match arp_requests.fetch_macs() {
            Ok(result) => result,
            Err(_) => HashMap::new()
        };
        self.mac_addresses = available_macs.clone();

        available_macs
    }
}


fn get_matches() -> ArgMatches<'static> {
    ClApp::new(BANNER)
        .author("Krakaw")
        .about("\nPoll ARP addresses and trigger an action when an mac changes.")
        .arg(
            Arg::with_name("list_interfaces")
                .short("l")
                .long("list")
                .help("List available interfaces including their index")
                .conflicts_with_all(&["interface", "monitor", "poll_time"])
        )
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .value_name("INTERFACE")
                .help("The interface on which the scan will be performed")
                .required_unless_one(&["list_interfaces"])
        )
        .arg(
            Arg::with_name("monitor_macs")
                .short("m")
                .long("monitor")
                .value_name("MAC_ADDRESSES")
                .multiple(true)
                .takes_value(true)
                .requires("interface")
                .help("List of mac addresses to monitor")
        )
        .arg(
            Arg::with_name("notify_on_new")
                .short("n")
                .long("new")
                .default_value_if("monitor_macs", None, "1")
                .requires("interface")
                .help("Notify when a never before seen mac is found")
        )
        .arg(
            Arg::with_name("poll_time")
                .short("p")
                .long("poll_time")
                .value_name("SECONDS")
                .default_value(DEFAULT_POLL)
                .help("How often should the network be polled")
        )
        .arg(
            Arg::with_name("debounce")
                .short("d")
                .long("debounce")
                .value_name("COUNT")
                .default_value(DEFAULT_DEBOUNCE)
                .help("Take the average count of available responses after [COUNT] requests")
        )
        .arg(
            Arg::with_name("lookup_macs")
                .short("o")
                .long("lookup_macs")
                .requires("interface")
                .help("Lookup the mac address vendors")
        )
        .arg(
            Arg::with_name("state_file")
                .short("s")
                .long("state_file")
                .value_name("FILE")
                .takes_value(true)
                .requires("interface")
                .help("Where to store the JSON state file")
        )
        .get_matches()
}