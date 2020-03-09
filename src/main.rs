#![feature(async_closure)]

mod actors;
mod assets;
mod control;
mod network;
mod types;

use actors::{CtrlServer, Database, LiveServer};
use anyhow::Error;
use std::time::Duration;
use tokio::time::timeout;

const TERMINATION_TIMEOUT: u64 = 10;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    log::info!("Tody.Chat - version {}", clap::crate_version!());

    log::debug!("Starting database actor...");
    let database = Database::new();
    let mut database_handle = meio::spawn(database);

    log::debug!("Starting Ctrl server...");
    let addr = ([127, 0, 0, 1], 3020).into();
    let ctrl_server = CtrlServer::new(addr);
    let mut ctrl_server_handle = meio::spawn(ctrl_server);

    log::debug!("Starting Live server...");
    let addr = ([127, 0, 0, 1], 3030).into();
    let live_server = LiveServer::new(addr);
    let mut live_server_handle = meio::spawn(live_server);

    log::info!("Press Ctrl-C to terminate.");
    tokio::signal::ctrl_c().await?;

    log::debug!("Terminating Live server...");
    live_server_handle.terminate();
    if let Err(err) = timeout(
        Duration::from_secs(TERMINATION_TIMEOUT),
        live_server_handle.join(),
    )
    .await
    {
        log::error!("Can't terminate Live server: {}", err);
    }

    log::debug!("Terminating Ctrl server...");
    ctrl_server_handle.terminate();
    if let Err(err) = timeout(
        Duration::from_secs(TERMINATION_TIMEOUT),
        ctrl_server_handle.join(),
    )
    .await
    {
        log::error!("Can't terminate Ctrl server: {}", err);
    }

    log::debug!("Terminating the database actor...");
    database_handle.terminate();
    if let Err(err) = timeout(Duration::from_secs(10), database_handle.join()).await {
        log::error!("Can't terminate the database actor: {}", err);
    }

    log::info!("Thank you for using Tody 🐦 App!");
    Ok(())
}
