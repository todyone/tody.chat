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
