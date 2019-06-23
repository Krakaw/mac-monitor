extern crate clap;
extern crate ipnetwork;
extern crate pnet;

#[macro_use]
extern crate prettytable;

use clap::{App, Arg, ArgMatches};

use prettytable::format;
use prettytable::Table;

use std::fs::File;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use ipnetwork::IpNetwork;
use pnet::datalink::{self, Channel, MacAddr, NetworkInterface};

use pnet::packet::arp::MutableArpPacket;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use std::str::FromStr;
use pnet::util::ParseMacAddrErr;
use std::error::Error;
use std::collections::HashMap;
use std::hash::Hash;

const BANNER: &str = "Arp Notify";
fn main() {
    let matches = App::new(BANNER)
        .about("\nPoll ARP addresses and trigger a web hook when an interface changes.")
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
                .value_name("MONITOR")
                .multiple(true)
                .takes_value(true)
                .requires("interface")
                .help("List of mac addresses to monitor")
        )
        .arg(
            Arg::with_name("list_interfaces")
                .short("l")
                .long("list")
                .help("List available interfaces including their index")
                .conflicts_with_all(&["interface", "monitor"])
        )
        .get_matches();

    if matches.is_present("list_interfaces") {
        println!("Listing interfaces:\n");
        let interfaces = datalink::interfaces();
        for interface in interfaces.iter() {
            println!("{}\n", interface);
        }
        std::process::exit(0);
    }

    let interface_match = |iface: &NetworkInterface| {
        return &iface.name == matches.value_of("interface").unwrap();
    };

    let interfaces = datalink::interfaces();
    let interface = interfaces
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

    println!("Using interface: {}\n", interface);

    let source_mac = interface.mac_address();
    let source_network = interface.ips.iter().find(|x| x.is_ipv4()).unwrap();
    let source_ip = source_network.ip();
    let arp_operation = ArpOperations::Request;


    let available_macs = match fetch_macs(interface) {
        Ok(result) => result,
        Err(_) => HashMap::new()
    };


    if matches.is_present("monitor_macs") {
        let monitor_macs: Vec<MacAddr> = matches
            .values_of("monitor_macs")
            .unwrap()
            .map(|x| MacAddr::from_str(x.clone()).unwrap())
            .collect();
        for monitor_mac in monitor_macs {
            if available_macs.contains_key(&monitor_mac) {
                println!("Mac Exists {} - {:?}", monitor_mac, available_macs.get(&monitor_mac));
            }else {
                println!("Mac Un-available {}", monitor_mac);
            }
        }
        std::process::exit(0);
    } else {


        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["host", "mac"]);

        let mut available_ips: HashMap<Ipv4Addr, MacAddr> = HashMap::new();
        for (mac, ip) in available_macs {
            available_ips.insert(ip, mac);
        }
        let mut ips:Vec<(&Ipv4Addr, &MacAddr)> = available_ips.iter().collect();
        ips.sort_by(|a,b| a.0.cmp(b.0));

//        let ips = ips
        for ( ip, mac) in ips {
            table.add_row(row![ip, mac]);
        }

        if table.len() > 0 {
            table.printstd();
        } else {
            println!("No hosts found...");
        }
    }

}

fn fetch_macs(interface: NetworkInterface) -> Result<HashMap<MacAddr, Ipv4Addr>, Box<Error>>{
    let source_mac = interface.mac_address();
    let source_network = interface.ips.iter().find(|x| x.is_ipv4()).unwrap();
    let source_ip = source_network.ip();
    let arp_operation = ArpOperations::Request;

    let target_mac = MacAddr::new(255, 255, 255, 255, 255, 255);
    // Channel for ARP replies.
    let (tx, rx): (Sender<(Ipv4Addr, MacAddr)>, Receiver<(Ipv4Addr, MacAddr)>) = mpsc::channel();

    recv_arp_packets(interface.clone(), tx);
    match source_network {
        &IpNetwork::V4(source_networkv4) => {
            for target_ipv4 in source_networkv4.iter() {
                match source_ip {
                    IpAddr::V4(source_ipv4) => {
                        send_arp_packet(
                            interface.clone(),
                            source_ipv4,
                            source_mac,
                            target_ipv4,
                            target_mac,
                            arp_operation,
                        );
                    }
                    e => panic!("Error while parsing to IPv4 address: {}", e),
                }
            }
        }
        e => panic!("Error while attempting to get network for interface: {}", e),
    }

    thread::sleep(Duration::from_secs(2));

    let mut results: HashMap<MacAddr, Ipv4Addr> = HashMap::new();
    loop {
        match rx.try_recv() {
            Ok((ipv4_addr, mac_addr)) => {
                results.insert(mac_addr, ipv4_addr);
            }
            Err(_) => break,
        }
    }
    Ok(results)
}

fn send_arp_packet(
    interface: NetworkInterface,
    source_ip: Ipv4Addr,
    source_mac: MacAddr,
    target_ip: Ipv4Addr,
    target_mac: MacAddr,
    arp_operation: ArpOperation,
) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(arp_operation);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    tx.send_to(ethernet_packet.packet(), Some(interface));
}

fn recv_arp_packets(interface: NetworkInterface, tx: Sender<(Ipv4Addr, MacAddr)>) {
    thread::spawn(move || {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };

        loop {
            match rx.next() {
                Ok(data) => {
                    let ethernet_packet = EthernetPacket::new(data).unwrap();
                    let ethernet_payload = ethernet_packet.payload();
                    if ArpPacket::new(ethernet_payload).is_some() {
                        let arp_packet = ArpPacket::new(ethernet_payload).unwrap();
                        let arp_reply_op = ArpOperation::new(2_u16);

                        if arp_packet.get_operation() == arp_reply_op {
                            let result: (Ipv4Addr, MacAddr) = (
                                arp_packet.get_sender_proto_addr(),
                                arp_packet.get_sender_hw_addr(),
                            );
                            tx.send(result).unwrap();
                        }
                    }
                }
                Err(e) => panic!("An error occurred while reading packet: {}", e),
            }
        }
    });
}
