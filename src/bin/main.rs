extern crate log;
extern crate simple_logger;

use log::{trace,info};

fn main() {
    simple_logger::init().unwrap();
    trace!("Starting I2P Client Daemon...");

    trace!("I2P Client Daemon Stopped.");
}