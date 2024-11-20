use crate::common::dto::{Database, InboundConnection, OutboundConnection, Runtime};
use crate::common::traits::ServerConfig;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
pub struct GSServer {
    pub name: String,
    pub blowfish_key: String,
    pub runtime: Option<Runtime>,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
}

impl ServerConfig for GSServer {
    fn load(file_name: &str) -> Self {
        let file = File::open(file_name)
            .unwrap_or_else(|e| panic!("Failed to open config file: {file_name}. Error: {e}"));
        let reader = BufReader::new(file);
        let config: GSServer = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!("Unable to parse {file_name}, the format is incorrect, {e}")
        });
        println!("Configuration ok, starting application: {}", config.name);
        config
    }
    fn from_string(conf: &str) -> Self {
        serde_yaml::from_str::<GSServer>(conf)
            .unwrap_or_else(|e| panic!("Unable to parse {conf}, the format is incorrect, {e}"))
    }

    fn runtime(&self) -> Option<&Runtime> {
        self.runtime.as_ref()
    }

    fn database(&self) -> &Database {
        &self.database
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientListener {
    pub connection: InboundConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginServerConnector {
    pub connection: OutboundConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listeners {
    pub clients: ClientListener,
    pub login_server: LoginServerConnector,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub timeout: u8,
}
