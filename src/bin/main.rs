extern crate log;
extern crate simple_logger;

use log::{trace,info};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Route, Packet, PacketType, NetworkId};
use nom::tag_cl;

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let mut client_1 = I2PClient::new();
    match client_1.init(true) {
        Ok(addr_1) => trace!("address 1: {}",addr_1.as_str()),
        Err(e) => trace!("{}",e)
    }

    let mut client_2 = I2PClient::new();
    let addr_2 = client_2.init(false);
    match client_1.init(true) {
        Ok(addr_2) => trace!("address 2: {}",addr_2.as_str()),
        Err(e) => trace!("{}",e)
    }

    let msg = format!("{}","Message from 1 to 2");
    let msg = msg.into_bytes();
    let env = Envelope::new(1, 2, msg);
    client_1.send(env);

    trace!("I2P Client Daemon Stopped.");
}