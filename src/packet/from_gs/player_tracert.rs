use crate::login_server::gs_thread::GSHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use async_trait::async_trait;
use crate::login_server::traits::PacketHandler;

#[derive(Clone, Debug)]
pub struct PlayerTracert {
    pub account: String,
    pub pc_ip: String,
    pub hop1: String,
    pub hop2: String,
    pub hop3: String,
    pub hop4: String,
}

#[async_trait]
impl ReadablePacket for PlayerTracert {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        let pc_ip = buffer.read_string();
        let hop1 = buffer.read_string();
        let hop2 = buffer.read_string();
        let hop3 = buffer.read_string();
        let hop4 = buffer.read_string();
        Some(Self {
            account: account_name,
            pc_ip,
            hop1,
            hop2,
            hop3,
            hop4,
        })
    }
}

#[async_trait]
impl GSHandle for PlayerTracert {
    async fn handle(&self, _: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        Ok(None)
    }
}
