mod controller;

use crate::opts::*;
use anyhow::Error;
use controller::Controller;
use protocol::request::actions::{self, InfoType};

pub struct Client {
    controller: Controller,
}

impl Client {
    pub async fn connect() -> Result<Self, Error> {
        let controller = Controller::connect("127.0.0.1:3020").await?;
        Ok(Self { controller })
    }

    pub async fn execute(&mut self, opts: Opts) -> Result<(), Error> {
        match opts.subcmd {
            SubCommand::User(cmd) => self.user(cmd).await,
            SubCommand::Channel(cmd) => self.channel(cmd).await,
            SubCommand::Run => {
                unreachable!();
            }
        }
    }

    pub async fn user(&mut self, cmd: UserCommand) -> Result<(), Error> {
        match cmd.subcmd {
            UserSubCommand::Create(user_create) => {
                // 1. Subscribe to user's changes/notifications/deltas
                let info = InfoType::User(Some(user_create.username.clone()));
                self.controller.subscribe(info.clone()).await?;
                // 2. Send a request to create a new user
                let action = actions::User::Create(user_create.username);
                self.controller.send(action).await?;
                // 3. Wait for the result
                // 4. Unsubscribe
                self.controller.unsubscribe(info.clone()).await?;
            }
        }
        Ok(())
    }

    pub async fn channel(&mut self, cmd: ChannelCommand) -> Result<(), Error> {
        match cmd.subcmd {
            ChannelSubCommand::Create(_) => {}
            ChannelSubCommand::Delete(_) => {}
            ChannelSubCommand::List => {}
        }
        Ok(())
    }
}
