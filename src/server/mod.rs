mod actors;
mod assets;

use crate::opts::Opts;
use actors::{CtrlServer, Engine, LiveServer};
use anyhow::Error;

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

        log::debug!("Starting engine actor...");
        let mut engine = Engine::start();

        log::debug!("Starting Ctrl server...");
        let addr = ([127, 0, 0, 1], 3020).into();
        let mut ctrl_server = CtrlServer::start(addr, engine.clone());

        log::debug!("Starting Live server...");
        let addr = ([127, 0, 0, 1], 3030).into();
        let mut live_server = LiveServer::start(addr, engine.clone());

        log::info!("Press Ctrl-C to terminate.");
        tokio::signal::ctrl_c().await?;

        live_server.terminate_with_timeout().await;
        ctrl_server.terminate_with_timeout().await;
        engine.terminate_with_timeout().await;

        log::info!("Thank you for using Tody 🐦 App!");
        Ok(())
    }
}
