extern crate log;
extern crate simple_logger;

use log::{trace,info};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Route, Packet, PacketType, NetworkId};
use nom::tag_cl;

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let mut client_1 = I2PClient::new(true, String::from("Alice"));
    trace!("address 1 ({}): {}",&client_1.local_dest.len(), &client_1.local_dest.as_str());

    let client_2 = I2PClient::new(false, String::from("Bob"));
    trace!("address 2 ({}): {}",client_2.local_dest.len(), client_2.local_dest.as_str());

    let msg = format!("{}","Message from 1 to 2");
    let msg = msg.into_bytes();
    let env = Envelope::new(0, 0, msg);
    let from = client_1.local_dest.clone();
    let to = client_2.local_dest.clone();
    let packet = Packet::new(1, PacketType::Data as u8, NetworkId::I2P as u8, from, to, Some(env) );
    client_1.send(packet);

    trace!("I2P Client Daemon Stopped.");
}