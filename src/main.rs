#![feature(async_closure)]

mod actors;

use actors::Server;
use failure::Error;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    log::info!("Tody.CRM - version {}", clap::crate_version!());

    log::debug!("Starting database actor...");

    log::debug!("Starting HTTP server...");
    let addr = ([127, 0, 0, 1], 3030).into();
    let server = Server::new(addr);
    let mut handle = meio::spawn(server);

    log::info!("Press Ctrl-C to terminate.");
    tokio::signal::ctrl_c().await?;

    log::debug!("Terminating HTTP server...");
    handle.terminate();
    let joiner = handle.join();
    if let Err(err) = timeout(Duration::from_secs(10), joiner).await {
        log::error!("Can't terminate the server: {}", err);
    }

    log::info!("Thank you for using Tody 🐦 App!");
    Ok(())
}
