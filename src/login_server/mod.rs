use crate::common::network;
use crate::common::traits::IpBan;
use crate::database::DBPool;
use crate::login_server::traits::{PacketHandler, TokioAsyncSocket};
use std::sync::Arc;
pub mod client_thread;
pub mod controller;
pub mod dto;
pub mod gs_thread;
mod message;
mod packet;
pub mod traits;

pub async fn main_loop<T, CFG, C>(config: Arc<CFG>, controller: Arc<C>, pool: DBPool)
where
    T: PacketHandler<ConfigType = CFG, ControllerType = C> + Send + Sync + 'static,
    C: IpBan,
{
    let conn_cfg = T::get_connection_config(&config);
    let listener =
        network::bind_addr(conn_cfg).unwrap_or_else(|_| panic!("Can not bind socket {conn_cfg:?}"));
    println!(
        "{} listening on {}",
        T::get_handler_name(),
        &listener.local_addr().unwrap()
    );
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                if let Ok(addr) = stream.peer_addr() {
                    println!(
                        "Incoming connection from {:?} ({:})",
                        addr.ip(),
                        T::get_handler_name()
                    );
                    if controller.is_ip_banned(&addr.ip().to_string()) {
                        eprint!("Ip is banned, skipping connection: {addr}"); //todo: maybe use EBPF?
                    } else {
                        let mut handler = T::new(stream, pool.clone(), controller.clone());
                        tokio::spawn(async move { handler.handle_client().await });
                    }
                }
            }
            Err(e) => {
                println!("Failed to accept connection: {e}");
            }
        }
    }
}
