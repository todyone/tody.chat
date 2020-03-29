#![feature(async_closure)]

mod actors;
mod assets;
mod client;
mod control;
mod db;
mod generators;
mod network;
mod opts;
mod server;

use anyhow::Error;
use clap::Clap;
use client::Client;
use opts::{Opts, SubCommand};
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    if let Some(SubCommand::Run) = opts.subcmd {
        Server::new(opts).run().await
    } else {
        Client::new(opts).run().await
    }
}
