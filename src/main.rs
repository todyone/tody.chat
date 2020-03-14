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
mod types;

use anyhow::Error;
use clap::Clap;
use client::Client;
use opts::Opts;
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    if opts.subcmd.is_some() {
        Client::new(opts).run().await
    } else {
        Server::new(opts).run().await
    }
}
