extern crate log;
extern crate simple_logger;

use log::{trace,info};
use i2p_client::I2PClient;

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");
    let mut client = I2PClient::new();
    client.init();

    trace!("I2P Client Daemon Stopped.");
}