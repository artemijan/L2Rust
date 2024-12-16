use l2_core::traits::handlers::PacketHandler;
use l2_core::packets::{
    ls_2_gs::RequestChars,
    error::PacketRun,
    gs_2_ls::ReplyChars,
};
use crate::{
    handlers::LoginHandler,
};
use async_trait::async_trait;
use entities::entities::character;
use tracing::instrument;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for RequestChars {
    type HandlerType = LoginHandler;

    #[instrument(skip_all)]
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        let chars =
            character::Model::find_characters_by_username(db_pool, &self.account_name).await?;
        let pack = ReplyChars::new(self.account_name.clone(), &chars);
        gs.send_packet(Box::new(pack)).await?;
        Ok(())
    }
}
