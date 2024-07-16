use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::{SendablePacket, ServerData, ServerStatus};
use crate::packet::LoginServerOpcodes;

#[derive(Debug, Clone)]
pub struct ServerList {
    pub buffer: SendablePacketBuffer,
    _servers: Vec<ServerData>,
    _last_server: i32,
    _chars_on_server: i32,
}

impl ServerList {
    pub fn new(
        servers: Vec<ServerData>,
        last_server: i32,
        total_chars_on_server: i32,
    ) -> ServerList {
        let mut sl = ServerList {
            buffer: SendablePacketBuffer::new(),
            _servers: servers,
            _last_server: last_server,
            _chars_on_server: total_chars_on_server,
        };
        sl.write_all().unwrap();
        sl
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::ServerList as i8)?;
        self.buffer.write_i8(self._servers.len() as i8)?;
        self.buffer.write_i8(self._last_server as i8)?;

        for server in self._servers.iter() {
            self.buffer.write_i8(server.server_id as i8)?;
            let ip_octets = server.get_ip_octets();
            self.buffer.write_i8(ip_octets[0] as i8)?;
            self.buffer.write_i8(ip_octets[1] as i8)?;
            self.buffer.write_i8(ip_octets[2] as i8)?;
            self.buffer.write_i8(ip_octets[3] as i8)?;
            self.buffer.write_i32(server.port)?;
            self.buffer.write_i8(server.age_limit as i8)?; // Age Limit 0, 15, 18
            if server.pvp {
                self.buffer.write_i8(0x01)?;
            } else {
                self.buffer.write_i8(0x00)?;
            }
            self.buffer.write_i16(server.current_players as i16)?;
            self.buffer.write_i16(server.max_players as i16)?;
            self.buffer
                .write_i8_from_bool(!matches!(server.status, ServerStatus::Down))?;
            self.buffer.write_i32(1024)?; // 1: Normal, 2: Relax, 4: Public Test, 8: No Label, 16: Character Creation Restricted, 32: Event, 64: Free
            self.buffer.write_i8_from_bool(server.brackets)?;
        }
        self.buffer.write_i16(0xA4)?; //unknown
        if self._chars_on_server > 0 {
            for server in self._servers.iter() {
                self.buffer.write_i8(server.server_id as i8)?;
                //todo here should be real count of chars on server
                self.buffer.write_i8(self._chars_on_server as i8)?;
            }
        }
        Ok(())
    }
}

impl SendablePacket for ServerList {
    fn get_bytes(&self) -> Vec<u8> {
        self.buffer.get_data()
    }
}
