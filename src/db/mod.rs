use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::Deserialize;
use tokio_postgres::NoTls;

pub mod calendar;
pub mod food;
pub mod shopping;

pub type ConnectionPool = bb8::Pool<PostgresConnectionManager<NoTls>>;
pub type Connection<'a> = bb8::PooledConnection<'a, PostgresConnectionManager<NoTls>>;

/// Configuration for the database connection.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl Config {
    pub fn to_config_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.dbname
        )
    }
}

/// generates a connection pool to the database.
/// This pool can hold multiple connections to the database.
pub async fn generate_pool(config: &Config) -> Result<ConnectionPool> {
    let manager = PostgresConnectionManager::new_from_stringlike(config.to_config_string(), NoTls)?;
    let pool = Pool::builder().build(manager).await?;

    Ok(pool)
}
