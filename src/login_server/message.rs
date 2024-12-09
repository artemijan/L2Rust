use std::time::SystemTime;
use tokio::sync::oneshot::Sender;

use crate::common::packets::common::{PacketType, SendablePacket};

#[derive(Debug)]
pub struct Request {
    pub response: Option<Sender<Option<(u8, PacketType)>>>,
    pub body: Option<Box<dyn SendablePacket>>,
    pub sent_at: SystemTime,
    pub id: String,
}
