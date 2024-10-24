use crate::login_server::client_thread::ClientHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::ClientHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::to_client::PlayOk;
use crate::packet::to_gs::RequestChars;
use async_trait::async_trait;
use crate::login_server::traits::PacketHandler;

#[derive(Clone, Debug)]
pub struct RequestGSLogin {
    pub s_key_1: i32,
    pub s_key_2: i32,
    pub server_id: u8,
}

impl ReadablePacket for RequestGSLogin {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        Some(RequestGSLogin {
            s_key_1: buffer.read_i32(),
            s_key_2: buffer.read_i32(),
            server_id: buffer.read_byte(),
        })
    }
}

#[async_trait]
impl ClientHandle for RequestGSLogin {
    async fn handle(
        &self,
        ch: &mut ClientHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let acc_name = ch.account_name.clone().unwrap();
        let lc = ch.get_lc();
        let message = Box::new(RequestChars::new(&acc_name));
        Ok(Some(Box::new(PlayOk::new(ch.get_session_key()))))
    }
}
