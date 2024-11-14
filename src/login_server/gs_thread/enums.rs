use crate::packet::common::PacketResult;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::{error, PlayerLoginFailReasons};
use strum::Display;

#[derive(Debug, Clone, Display)]
pub enum GS {
    Initial,
    Connected,
    BfConnected,
    Authed,
}

impl GS {
    pub fn transition_to(&mut self, desired_state: &GS) -> PacketResult {
        match (&self, desired_state) {
            (Self::Initial, Self::Connected) => *self = Self::Connected,
            (Self::Connected, Self::BfConnected) => *self = Self::BfConnected,
            (Self::BfConnected, Self::Authed) => *self = Self::Authed,
            _ => {
                return Err(error::PacketRun {
                    msg: Some(format!(
                        "Can not upgrade connection state for game server from {self}, to {desired_state}"
                    )),
                    response: Some(Box::new(PlayerLogin::new(PlayerLoginFailReasons::ReasonNotAuthed))),
                });
            }
        }
        Ok(None)
    }
}
