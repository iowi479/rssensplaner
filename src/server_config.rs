use crate::{bring, db};
use serde::Deserialize;
use std::net::SocketAddr;

/// Config holds all the configurations needed to start the API.
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub server: Config,
    pub database: db::Config,
    pub bring: bring::Config,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host.parse().unwrap(), self.port)
    }
}
