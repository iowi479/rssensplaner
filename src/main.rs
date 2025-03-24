use anyhow::Result;

pub mod bring;
pub mod calendar;
pub mod db;
pub mod food;
pub mod server;
pub mod server_config;
pub mod shopping;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let config = server_config::load_config()?;

    // Initialize and run the server
    let listener = tokio::net::TcpListener::bind(config.server.get_addr()).await?;
    let app = server::init_rest_api(config).await;
    axum::serve(listener, app).await?;

    eprintln!("Server closed unexpectedly");
    Ok(())
}
