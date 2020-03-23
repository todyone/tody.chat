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
                    controller.create_user(cmd.username, cmd.password).await?;
                }
            },
            Some(SubCommand::Channel(channel_command)) => match channel_command.subcmd {
                ChannelSubCommand::Create(cmd) => {
                    controller.create_channel(cmd.channel, cmd.username).await?;
                }
                ChannelSubCommand::List => {
                    println!("Channels:");
                    let channels = controller.get_channels().await?;
                    for channel in channels {
                        println!("{}", channel);
                    }
                }
            },
            _ => {
                unreachable!();
            }
        }
        Ok(())
    }
}
