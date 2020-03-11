// TODO: Rewrite this module to fully async
// when SQLite crates will support that.

use crate::db::{Dba, User};
use crate::types::{Password, Username};
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, StreamExt};
use meio::{wrapper, Actor, Address, Context, Interaction, InteractionHandler};
use tokio::task::block_in_place as wait;

wrapper!(Database for DatabaseActor);

impl Database {
    pub fn start() -> Self {
        let actor = DatabaseActor { dba: None };
        meio::spawn(actor)
    }
}

/*
{
    pub async fn create_user(&mut self, username: Username) -> Result<(), Error> {
        self.send(Msg::CreateUser { username }).await
    }

    pub async fn set_password(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), Error> {
        self.send(Msg::SetPassword { username, password }).await
    }

    pub async fn get_user(&mut self, username: Username) -> Result<User, Error> {
        self.send(Msg::GetUser { username }).await?;
        todo!();
    }
}
*/

pub struct DatabaseActor {
    dba: Option<Dba>,
}

pub struct CreateUser {
    username: Username,
}

impl Interaction for CreateUser {
    type Output = ();
}

struct SetPassword {
    username: Username,
    password: Password,
}

impl Interaction for SetPassword {
    type Output = ();
}

struct GetUser {
    username: Username,
}

impl Interaction for GetUser {
    type Output = User;
}

#[async_trait]
impl Actor for DatabaseActor {
    type Interface = Database;

    fn generic_name() -> &'static str {
        "Database"
    }

    async fn initialize(&mut self, _address: Address<Self>) -> Result<(), Error> {
        let dba = Dba::open()?;
        self.dba = Some(dba);
        wait(|| self.dba().initialize())?;
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<CreateUser> for DatabaseActor {
    async fn handle(&mut self, input: CreateUser) -> Result<(), Error> {
        wait(|| self.dba().create_user(input.username))?;
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<SetPassword> for DatabaseActor {
    async fn handle(&mut self, input: SetPassword) -> Result<(), Error> {
        // TODO: Protect password
        wait(|| self.dba().set_password(input.username, input.password))?;
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<GetUser> for DatabaseActor {
    async fn handle(&mut self, input: GetUser) -> Result<User, Error> {
        todo!();
    }
}

/// DatabaseActor routines.
impl DatabaseActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
