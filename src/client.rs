use crate::control::Controller;
use crate::opts::*;
use anyhow::Error;

pub struct Client {
    opts: Opts,
}

impl Client {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }

    pub async fn run(self) -> Result<(), Error> {
        let mut controller = Controller::connect("127.0.0.1:3020").await?;
        match self.opts.subcmd {
            Some(SubCommand::User(user_command)) => match user_command.subcmd {
                UserSubCommand::Create(user_create_command) => {
                    controller
                        .create_user(user_create_command.username, user_create_command.password)
                        .await?;
                }
            },
            _ => {
                unreachable!();
            }
        }
        Ok(())
    }
}
