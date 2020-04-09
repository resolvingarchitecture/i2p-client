extern crate log;
extern crate simple_logger;

use log::{trace,warn};
use std::env;
use std::thread;
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Packet, PacketType, NetworkId};

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let args: Vec<String> = env::args().collect();
    let seed = &args[1];
    trace!("Seed: {}", seed);

    let alias = String::from("Bob");
    let mut client = I2PClient::new(true, alias.clone());
    trace!("address ({}): {}", &client.local_dest.len(), &client.local_dest.as_str());

    let from = client.local_dest.clone();

    for i in 1..3 {
        let env = Envelope::new(0, 0, format!("Message {} from Bob to seed (Alice)", i).into_bytes());
        let packet_send = Packet::new(
            i,
            PacketType::Data as u8,
            NetworkId::I2P as u8,
            from.clone(),
            seed.clone(),
            Some(env));
        trace!("Sending msg({})...",i);
        client.send(packet_send);
    }

    for _i in 1..3 {
        match client.receive() {
            Ok(packet) => {
                if packet.envelope.is_some() {
                    trace!("msg: {}", String::from_utf8(packet.envelope.unwrap().msg).unwrap().as_str());
                }
            },
            Err(e) => warn!("{}", e)
        }
    }

    // client_alice.shutdown();

    trace!("I2P Client Daemon Stopped.");
}