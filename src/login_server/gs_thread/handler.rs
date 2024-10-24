use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpStream;
use sqlx::AnyPool;
use tokio::sync::{mpsc, Mutex, Notify, RwLock};
use std::collections::HashMap;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use anyhow::{bail, Error};
use openssl::error::ErrorStack;
use tokio::io::AsyncWriteExt;
use crate::common::dto::config::{Connection, Server};
use crate::common::errors::Packet;
use crate::common::message::Request;
use crate::crypt::new::Crypt;
use crate::crypt::rsa::ScrambledRSAKeyPair;
use crate::login_server::gs_thread::connection_state::GS;
use crate::login_server::controller::Login;
use crate::login_server::traits::{PacketHandler, Shutdown};
use crate::packet::common::{GSHandle, PacketResult, SendablePacket};
use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::error;
use crate::packet::gs_factory::build_gs_packet;
use crate::packet::to_gs::InitLS;


#[derive(Debug, Clone)]
pub struct GSHandler {
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    shutdown_listener: Arc<Notify>,
    lc: Arc<Login>,
    db_pool: AnyPool,
    key_pair: ScrambledRSAKeyPair,
    blowfish: Crypt,
    connection_state: GS,
    pub server_id: Option<u8>,
    unhandled_messages: Arc<RwLock<HashMap<String, Request>>>,
}

impl GSHandler {
    pub fn set_blowfish_key(&mut self, new_bf_key: &[u8]) {
        self.blowfish = Crypt::from_u8_key(new_bf_key);
    }
    pub async fn start_channel(&self) {
        let (rx, mut tx) = mpsc::channel::<Request>(100);
        self.lc.connect_gs(self.server_id.unwrap(), rx).await;
        let gs_handler_clone = Arc::new(self.clone());
        tokio::spawn(async move {
            loop {
                if let Some(request) = tx.recv().await {
                    let mut income_messages = gs_handler_clone.unhandled_messages.write().await;
                    //the message has been sent already, there is no sense to do it twice
                    if income_messages.contains_key(&request.id) {
                        let _ = request.response.send(None);
                    } else {
                        // send packet later, now we only remember it
                        let req_bytes = request.body.get_bytes();
                        if gs_handler_clone.send_bytes(req_bytes).await.is_ok() {
                            income_messages.insert(request.id.clone(), request);
                        } else {
                            let _ = request.response.send(None);
                        }
                    }
                }
            }
        });
    }
    pub fn set_connection_state(&mut self, state: &GS) -> PacketResult {
        self.connection_state.transition_to(state)
    }
    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), Packet> {
        self.blowfish.decrypt(data)
    }

    pub fn decrypt_rsa(&self, data: &mut [u8]) -> Result<Vec<u8>, ErrorStack> {
        self.key_pair.decrypt_data(data)
    }
}

impl Shutdown for GSHandler {
    fn get_shutdown_listener(&self) -> Arc<Notify> {
        self.shutdown_listener.clone()
    }

    fn shutdown(&self) {
        self.shutdown_listener.notify_one();
    }
}

#[async_trait]
impl PacketHandler for GSHandler {
    fn get_handler_name() -> String {
        "Game server handler".to_string()
    }
    fn get_connection_config(cfg: &Server) -> &Connection {
        &cfg.listeners.game_servers.connection
    }
    fn get_lc(&self) -> &Arc<Login> {
        &self.lc
    }

    fn new(mut stream: TcpStream, db_pool: AnyPool, lc: Arc<Login>) -> Self {
        let (tcp_reader, tcp_writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(tcp_writer));
        let reader = Arc::new(Mutex::new(tcp_reader));
        let cfg = lc.get_config();
        GSHandler {
            tcp_reader: reader,
            tcp_writer: writer,
            db_pool,
            shutdown_listener: Arc::new(Notify::new()),
            key_pair: lc.get_random_rsa_key_pair(),
            blowfish: Crypt::from_u8_key(cfg.blowfish_key.as_bytes()),
            connection_state: GS::Initial,
            lc,
            server_id: None,
            unhandled_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn on_connect(&mut self) -> Result<(), Packet> {
        println!(
            "Game server connected: {:?}",
            self.tcp_reader.lock().await.peer_addr().unwrap()
        );
        self.connection_state = GS::Connected;
        let init_packet = Box::new(InitLS::new(self.key_pair.get_modulus()));
        self.send_packet(init_packet).await?;
        Ok(())
    }

    async fn on_disconnect(&mut self) {
        println!(
            "Game server disconnected: ID ({:})",
            self.server_id.unwrap_or_default()
        );
        if let Some(server_id) = self.server_id {
            let lc = self.get_lc();
            lc.remove_gs(server_id).await;
        }
    }

    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        None
    }

    async fn send_packet(&mut self, mut packet: Box<dyn SendablePacket>) -> Result<(), Error> {
        let mut buffer = packet.get_buffer_mut();
        buffer.write_i32(0)?;
        let padding = (buffer.get_size() - 2) % 8;
        if padding != 0 {
            for _ in padding..8 {
                buffer.write_u8(0)?;
            }
        }
        self.send_bytes(packet.get_bytes()).await
    }
    async fn send_bytes(&self, mut bytes: Vec<u8>) -> Result<(), Error> {
        let size = bytes.len();
        Crypt::append_checksum(&mut bytes[2..size]);
        self.blowfish.crypt(&mut bytes[2..size]);
        self.get_stream_writer_mut()
            .await
            .lock()
            .await
            .write_all(&bytes)
            .await?;
        Ok(())
    }

    async fn on_receive_bytes(&mut self, _: usize, bytes: &mut [u8]) -> Result<(), Error> {
        self.blowfish.decrypt(bytes)?;
        if !Crypt::verify_checksum(bytes) {
            bail!("Can not verify check sum.")
        }
        let handler = build_gs_packet(bytes).ok_or_else(|| Packet::ClientPacketNotFound {
            opcode: bytes[0] as usize,
        })?;
        let resp = handler.handle(self).await;
        self.handle_result(resp).await
    }
}