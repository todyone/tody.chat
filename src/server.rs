use crate::actors::{CtrlServer, Database, LiveServer};
use crate::opts::Opts;
use anyhow::Error;
use meio::{Actor, Wrapper};
use std::time::Duration;
use tokio::time::timeout;

pub struct Server {
    opts: Opts,
}

impl Server {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }

    pub async fn run(self) -> Result<(), Error> {
        env_logger::try_init()?;
        log::info!("Tody.Chat - version {}", clap::crate_version!());

        log::debug!("Starting database actor...");
        let database = Database::new();
        let mut database_handle = meio::spawn(database);

        log::debug!("Starting Ctrl server...");
        let addr = ([127, 0, 0, 1], 3020).into();
        let ctrl_server = CtrlServer::new(addr, database_handle.clone());
        let mut ctrl_server_handle = meio::spawn(ctrl_server);

        log::debug!("Starting Live server...");
        let addr = ([127, 0, 0, 1], 3030).into();
        let live_server = LiveServer::new(addr, database_handle.clone());
        let mut live_server_handle = meio::spawn(live_server);

        log::info!("Press Ctrl-C to terminate.");
        tokio::signal::ctrl_c().await?;

        live_server_handle.terminate_with_timeout().await;
        ctrl_server_handle.terminate_with_timeout().await;
        database_handle.terminate_with_timeout().await;

        log::info!("Thank you for using Tody 🐦 App!");
        Ok(())
    }
}
