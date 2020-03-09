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
                UserSubCommand::Create(cmd) => {
                    controller.create_user(cmd.username.clone()).await?;
                    controller.set_password(cmd.username, cmd.password).await?;
                }
            },
            _ => {
                unreachable!();
            }
        }
        Ok(())
    }
}
