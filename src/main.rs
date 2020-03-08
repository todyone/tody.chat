#![feature(async_closure)]

mod actors;
mod assets;
mod control;
mod network;
mod types;

use actors::{Database, LiveServer};
use anyhow::Error;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    log::info!("Tody.Chat - version {}", clap::crate_version!());

    log::debug!("Starting database actor...");
    let database = Database::new();
    let mut database_handle = meio::spawn(database);

    log::debug!("Starting HTTP server...");
    let addr = ([127, 0, 0, 1], 3030).into();
    let server = LiveServer::new(addr);
    let mut server_handle = meio::spawn(server);

    log::info!("Press Ctrl-C to terminate.");
    tokio::signal::ctrl_c().await?;

    log::debug!("Terminating HTTP server...");
    server_handle.terminate();
    if let Err(err) = timeout(Duration::from_secs(10), server_handle.join()).await {
        log::error!("Can't terminate the server: {}", err);
    }

    log::debug!("Terminating the database actor...");
    database_handle.terminate();
    if let Err(err) = timeout(Duration::from_secs(10), database_handle.join()).await {
        log::error!("Can't terminate the database actor: {}", err);
    }

    log::info!("Thank you for using Tody 🐦 App!");
    Ok(())
}
