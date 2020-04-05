use clap::Clap;
use protocol::types::*;

#[derive(Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    #[clap(name = "run", about = "Start a server")]
    Run,
    #[clap(name = "user", about = "Manage user accounts")]
    User(UserCommand),
    #[clap(name = "channel", about = "Manage channels")]
    Channel(ChannelCommand),
}

#[derive(Clap)]
pub struct UserCommand {
    #[clap(subcommand)]
    pub subcmd: UserSubCommand,
}

#[derive(Clap)]
pub enum UserSubCommand {
    #[clap(name = "create", about = "Create a new user")]
    Create(UserCreateCommand),
}

#[derive(Clap)]
pub struct UserCreateCommand {
    pub username: Username,
    pub password: Password,
}

#[derive(Clap)]
pub struct ChannelCommand {
    #[clap(subcommand)]
    pub subcmd: ChannelSubCommand,
}

#[derive(Clap)]
pub enum ChannelSubCommand {
    #[clap(name = "create", about = "Create a new channel")]
    Create(ChannelCreateCommand),
    #[clap(name = "list", about = "List of channels")]
    List,
    #[clap(name = "delete", about = "Delete a channel")]
    Delete(ChannelDeleteCommand),
}

#[derive(Clap)]
pub struct ChannelCreateCommand {
    pub channel: ChannelName,
    pub username: Username,
}

#[derive(Clap)]
pub struct ChannelDeleteCommand {
    pub channel: ChannelName,
}
