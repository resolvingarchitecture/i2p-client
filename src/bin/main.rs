extern crate log;
extern crate simple_logger;

use log::{trace,info};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Route};

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let mut client = I2PClient::new();
    client.init();
    let msg = format!("{}","Messge from 1 to 2");
    let msg = msg.into_bytes();
    let env = Envelope::new(1, 2, msg);

    trace!("I2P Client Daemon Stopped.");
}