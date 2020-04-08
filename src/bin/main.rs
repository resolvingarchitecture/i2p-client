extern crate log;
extern crate simple_logger;

use log::{trace,info,warn};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Route, Packet, PacketType, NetworkId};
use nom::tag_cl;
use std::error::Error;

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let mut client = I2PClient::new(true, String::from("Alice"));
    trace!("address ({}): {}",&client.local_dest.len(), &client.local_dest.as_str());

    let env = Envelope::new(0, 0, format!("{}","Message from me to me").into_bytes());
    let from = client.local_dest.clone();
    let to = client.local_dest.clone();
    let packet_send = Packet::new(
        1,
        PacketType::Data as u8,
        NetworkId::I2P as u8,
        from,
        to,
        Some(env));
    client.send(packet_send);
    match client.receive() {
        Ok(packet) => {
            if packet.envelope.is_some() {
                trace!("msg: {}", String::from_utf8(packet.envelope.unwrap().msg).unwrap().as_str());
            }
        },
        Err(e) => warn!("{}",e.to_string())
    }
    trace!("I2P Client Daemon Stopped.");
}