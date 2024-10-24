use crate::login_server::gs_thread::GSHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerInGame {
    accounts: Vec<String>,
}

impl ReadablePacket for PlayerInGame {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i16();
        let mut accounts: Vec<String> = vec![];
        for _ in 0..size {
            let st = buffer.read_string();
            accounts.push(st);
        }
        Some(PlayerInGame { accounts })
    }
}

#[async_trait]
impl GSHandle for PlayerInGame {
    async fn handle(&self, _gs: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        Ok(None)
    }
}
