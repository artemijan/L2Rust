use async_trait::async_trait;
use tracing::info;
use l2_core::packets::ls_2_gs::KickPlayer;
use l2_core::packets::error::PacketRun;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for KickPlayer {
    type HandlerType = LoginHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> Result<(), PacketRun> {
        info!("TODO: Implement KickPlayer packet handler");
        Ok(())
    }
}
