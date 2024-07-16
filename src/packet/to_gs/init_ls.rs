use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;
use crate::packet::LoginServerOpcodes;

#[derive(Debug)]
pub struct InitLS {
    pub buffer: SendablePacketBuffer,
    public_key: Vec<u8>,
}

impl InitLS {
    pub const PROTOCOL_REVISION: i32 = 0x0106;
    pub fn new(public_key: Vec<u8>) -> Self {
        let mut init_ls = InitLS {
            buffer: SendablePacketBuffer::new(),
            public_key,
        };
        init_ls.write_all().unwrap();
        init_ls
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(LoginServerOpcodes::Init as u8)?;
        self.buffer.write_i32(Self::PROTOCOL_REVISION)?; // LS protocol revision
        self.buffer.write_i32(self.public_key.len() as i32)?; // key length
        self.buffer.write_bytes(self.public_key.clone())?; // RSA Public Key
        Ok(())
    }
}

impl SendablePacket for InitLS {
    fn get_bytes(&self) -> Vec<u8> {
        self.buffer.get_data()
    }
}
