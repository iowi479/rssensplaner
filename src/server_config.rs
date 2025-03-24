use crate::{bring, db};
use anyhow::{Context, Result};
use config::{File, FileFormat};
use serde::Deserialize;
use std::env;
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

/// Tries to load the configuration from the file `config.toml`.
/// If the file does not exist, it tries to load the configuration from the environment.
///
/// # Variables
/// - `RSSESPLANER_HOST`: The host the server should listen on.
/// - `RSSESPLANER_PORT`: The port the server should listen on.
///
/// - `RSSESPLANER_DB_HOST`: The host of the database.
/// - `RSSESPLANER_DB_PORT`: The port of the database.
/// - `RSSESPLANER_DB_USER`: The user to connect to the database.
/// - `RSSESPLANER_DB_PASSWORD`: The password to connect to the database.
/// - `RSSESPLANER_DB_NAME`: The name of the database.
///
/// - `RSSESPLANER_BRING_EMAIL`: The email to login to Bring! API.
/// - `RSSESPLANER_BRING_PASSWORD`: The password to login to Bring! API.
///
/// Fails if the file cannot be read and any of the variables are not set.
pub fn load_config() -> Result<ServerConfig> {
    // Load the configuration file
    let file_config_res = config::Config::builder()
        .add_source(File::new("config.toml", FileFormat::Toml))
        .build();

    if let Ok(file_config) = file_config_res {
        let config = file_config.try_deserialize::<ServerConfig>()?;
        return Ok(config);
    }

    // could not open the file, try to load the configuration from the environment
    let config = load_from_env()?;

    Ok(config)
}

/// Tries to loads the whole `ServerConfig` from the environment.
fn load_from_env() -> Result<ServerConfig> {
    let config = ServerConfig {
        server: Config {
            host: env::var("RSSESPLANER_HOST").with_context(|| "RSSESPLANER_HOST not set")?,
            port: env::var("RSSESPLANER_PORT")
                .with_context(|| "RSSESPLANER_PORT not set")?
                .parse()?,
        },
        database: db::Config {
            host: env::var("RSSESPLANER_DB_HOST").with_context(|| "RSSESPLANER_DB_HOST not set")?,
            port: env::var("RSSESPLANER_DB_PORT")
                .with_context(|| "RSSESPLANER_DB_PORT not set")?
                .parse()?,
            user: env::var("RSSESPLANER_DB_USER").with_context(|| "RSSESPLANER_DB_USER not set")?,
            password: env::var("RSSESPLANER_DB_PASSWORD")
                .with_context(|| "RSSESPLANER_DB_PASSWORD not set")?,
            dbname: env::var("RSSESPLANER_DB_NAME")
                .with_context(|| "RSSESPLANER_DB_NAME not set")?,
        },
        bring: bring::Config {
            email: env::var("RSSESPLANER_BRING_EMAIL")
                .with_context(|| "RSSESPLANER_BRING_EMAIL not set")?,
            password: env::var("RSSESPLANER_BRING_PASSWORD")
                .with_context(|| "RSSESPLANER_BRING_PASSWORD not set")?,
        },
    };

    Ok(config)
}
