use log::{trace,info,warn};
use ra_common::models::{Network, Packet};
use i2p::sam::{DEFAULT_API,SamConnection,Session,SessionStyle};
// use crate::sam::{DEFAULT_API,SamConnection,Session,SessionStyle};
// pub mod sam;
// pub mod parsers;

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
                info!("dest: {}",conn.naming_lookup("1m5.i2p").unwrap());
            },
            Err(e) => {
                warn!("Error: {}",e.to_string());
            }
        }
        // match Session::create(DEFAULT_API,
        //                       "PwyBbml~AOWQsrZ61y2UqdXSuGGy5E6whh93k1dzIuvBRWr5K09X3Pzpt1wBFffA8Y02vAyrX8d2rCjrRcYFaxhbkummyUqeN7zdeqqZ3NDbbm-qyZH7tBEv-QwjxA18hxG~x~9-tP-ixiYMNDOMs5FhPThZkb-RFpY1HgTB19DbGL35KqYWtmuZG1~ZYy99c~u3kDEOroj1Jm5vAQFXbemlUOLrUZkyV4b0UkLqe1KRMWSOdGb8QL6PraBLx3QFvVJfrEji~u~8ztEIOZovvy9xOf52SvTsUQc-OVaFU7xPnvbxMlnW43JgT9w0RG2~Rh6UhKvmGcnUb5yVhdmujXY9gJ98bJ--VhBBAzGbPBxPN0~WqQ2rzEQ7x-~A2SBGhYfRhxQTFiC-n~cQehok6zMEcm0gSsD80mYdZWFskymm-OqXTd~AxoXkuX6xdv3l2TYJz6TEl-5TvTw38v7LD8WeDiC001YCwGKenxc4VQfi2xzXW9hAuoTuPww8L5yvBQAEAAcAAA==",
        //                       "1m5.i2p",
        //                       SessionStyle::Datagram) {
        //     Ok(session) => {
        //         let session = Box::new(session);
        //         info!("IP: {}",session.sam_api().unwrap().ip().to_string())
        //     },
        //     Err(err) => {
        //         warn!("Error: {}",err.to_string());
        //     }
        // }
    }
}

impl Network for I2PClient {
    fn handle(&mut self, packet: &mut Packet) {
        info!("Handling incoming packet id={}",packet.id);

    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
