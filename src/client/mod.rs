mod controller;

use crate::opts::Opts;
use anyhow::Error;
use controller::Controller;

pub struct Client {
    opts: Opts,
}

impl Client {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }

    pub async fn run(self) -> Result<(), Error> {
        let mut controller = Controller::connect("127.0.0.1:3020").await?;
        Ok(())
    }
}
