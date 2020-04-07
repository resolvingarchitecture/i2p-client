
use log::{info,warn};

use ra_common::models::{Packet, Service, Envelope};
use i2p::sam::{SamConnection, Session, SessionStyle, DEFAULT_API, StreamConnect};

pub enum ClientType {
    Local    = 0,
    Embedded = 1,
}

pub struct I2PClient {

}

impl I2PClient {
    pub fn new() -> I2PClient {
        I2PClient {}
    }
    pub fn init(&mut self) {
        info!("{}","Initializing I2P Client...");

        match SamConnection::connect(DEFAULT_API) {
            Ok(mut conn) => {
                match conn.naming_lookup("ME") {
                    Ok(local_dest) => info!("local: {}",local_dest),
                    Err(e) => warn!("Error: {}",e.to_string())
                }
            },
            Err(e) => warn!("Error: {}",e.to_string())
        }

        info!("{}","I2P Client initialized.");
    }

    // Handle incoming packets
    pub fn handle(&mut self, packet: &mut Packet) {
        info!("Handling incoming packet id={}",packet.id);

    }

    // Send out Envelope as a Packet
    pub fn send(&mut self, env: Envelope) {
        match SamConnection::connect(DEFAULT_API) {
            Ok(mut conn) => {
                // TODO: get address from provided Envelope
                let dest = conn.naming_lookup("1m5.i2p").unwrap();
                info!("dest: {}",dest);
            },
            Err(e) => {
                warn!("Error: {}",e.to_string());
            }
        }
        match Session::create(DEFAULT_API,
                              "1m5.i2p",
                              "1m5",
                              SessionStyle::Datagram) {
            Ok(session) => {
                info!("IP: {}",session.sam_api().unwrap().ip().to_string())
            },
            Err(err) => {
                warn!("Error: {}",err.to_string());
            }
        }

        // let mut connection = StreamConnect::new(DEFAULT_STREAM_API, "1m5.i2p", 8000, "1m5").unwrap();
        // let local_addr = connection.local_addr().unwrap();
        // let peer_addr = connection.peer_addr().unwrap();
        // info!("Local addr: {}:{}",local_addr.0,local_addr.1);
        // info!("Peer addr: {}:{}",peer_addr.0,peer_addr.1);
    }
}

impl Service for I2PClient {
    fn operate(&mut self, operation: u8, env: Envelope) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
