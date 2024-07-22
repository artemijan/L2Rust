use crate::common::errors::PacketErrors;
use crate::common::errors::PacketErrors::UnableToHandleClientPacket;
use crate::login_server::controller::LoginController;
use crate::packet::common::SendablePacket;
use crate::packet::error::PacketRunError;
use anyhow::{bail, Error};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex;
use tokio::time::timeout;

pub mod controller;
pub mod gs_handler;
pub mod ls_handler;
pub mod event_loops;

pub const PACKET_SIZE_BYTES: usize = 2;

#[async_trait]
pub trait PacketHandler {
    fn get_handler_name() -> String;
    fn get_lc(&self) -> &Arc<LoginController>;

    async fn on_connect(&mut self) -> Result<(), PacketErrors>;

    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>>;
    fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>>;
    fn get_timeout(&self) -> Option<u64> {
        None
    }

    async fn send_packet(&mut self, packet: Box<dyn SendablePacket>) -> Result<(), Error> {
        self.send_bytes(packet.get_bytes()).await
    }
    async fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), Error>;
    async fn on_receive_bytes(
        &mut self,
        packet_size: usize,
        bytes: &mut [u8],
    ) -> Result<(), Error>;

    async fn read_packet(&mut self) -> anyhow::Result<(usize, Vec<u8>)> {
        let mut size_buf = [0; PACKET_SIZE_BYTES];
        let mut socket = self.get_stream_reader_mut().lock().await;
        socket.read_exact(&mut size_buf).await?;
        let size = (u16::from_le_bytes(size_buf) as usize) - PACKET_SIZE_BYTES;
        // Read the body of the packet based on the size
        let mut body = vec![0; size];
        socket.read_exact(&mut body).await?;
        Ok((size, body))
    }
    
    async fn handle_result(
        &mut self,
        resp: Result<Option<Box<dyn SendablePacket>>, PacketRunError>,
    ) -> Result<(), anyhow::Error> {
        match resp {
            Ok(result) => {
                if let Some(packet) = result {
                    self.send_packet(packet).await?;
                }
            }
            Err(e) => {
                if let Some(packet) = e.response {
                    self.get_stream_writer_mut()
                        .lock()
                        .await
                        .write_all(packet.get_bytes().as_slice())
                        .await?;
                } else if let Some(msg) = e.msg {
                    bail!(UnableToHandleClientPacket { msg })
                }
            }
        }
        Ok(())
    }
    
    async fn handle_client(&mut self) {
        let client_addr = self
            .get_stream_reader_mut()
            .lock()
            .await
            .peer_addr()
            .unwrap();
        if let Err(e) = self.on_connect().await {
            eprintln!(
                "{}: Disconnecting client. Error: {}",
                Self::get_handler_name(),
                e
            );
            return;
        }
        loop {
            let tm_out = self.get_timeout();
            let read_future = self.read_packet();
            let read_result = match tm_out {
                Some(t_out) => {
                    let t_duration = Duration::from_secs(t_out);
                    match timeout(t_duration, read_future).await {
                        Ok(r) => r,
                        Err(_) => {
                            println!(
                                "{}: No data received for {:?}. Dropping connection.",
                                Self::get_handler_name(),
                                t_duration
                            );
                            break;
                        }
                    }
                }
                None => read_future.await,
            };
            match read_result {
                Ok((0, _)) => {
                    println!(
                        "{}: Client disconnected: {:?}",
                        Self::get_handler_name(),
                        client_addr
                    );
                    break;
                }
                Ok((bytes_read, mut data)) => {
                    if let Err(e) = self.on_receive_bytes(bytes_read, &mut data).await {
                        eprintln!(
                            "{}: Disconnecting client {}, because error occurred {}",
                            Self::get_handler_name(),
                            client_addr,
                            e
                        );
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to read data from client: {}",
                        Self::get_handler_name(),
                        e
                    );
                    break;
                }
            }
        }
    }
}
