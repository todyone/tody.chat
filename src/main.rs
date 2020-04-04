use anyhow::Error;
use clap::Clap;
use tody_chat::opts::{Opts, SubCommand};
use tody_chat::{Client, Server};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    if let SubCommand::Run = opts.subcmd {
        Server::new(opts).run().await
    } else {
        Client::new(opts).run().await
    }
}
