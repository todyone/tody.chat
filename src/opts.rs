use clap::Clap;

#[derive(Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Clap)]
pub enum SubCommand {
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
    pub username: String,
    pub password: String,
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
}

#[derive(Clap)]
pub struct ChannelCreateCommand {
    pub channel: String,
    pub username: String,
}
