use ra_common::models::{Network, Packet};

pub struct I2PClient {

}

impl I2PClient {
    pub fn new() -> Box<I2PClient> {
        Box::new(I2PClient {

        })
    }
    pub fn init(&mut self) {

    }
}

impl Network for I2PClient {
    fn handle(&mut self, packet: &mut Packet) {
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
