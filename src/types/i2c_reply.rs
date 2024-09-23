/// An I2C reply.
#[derive(Debug, Default)]
pub struct I2CReply {
    pub address: u8,
    pub register: u8,
    pub data: Vec<u8>,
}
