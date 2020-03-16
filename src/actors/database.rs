// TODO: Rewrite this module to fully async
// when SQLite crates will support that.

use crate::db::{Dba, Session, User};
use crate::types::{Id, Password, Username};
use anyhow::Error;
use async_trait::async_trait;
use meio::{wrapper, Actor, Address, Interaction, InteractionHandler};
use protocol::Key;
use tokio::task::block_in_place as wait;

wrapper!(Database for DatabaseActor);

impl Database {
    pub fn start() -> Self {
        let actor = DatabaseActor { dba: None };
        meio::spawn(actor)
    }

    pub async fn create_user(&mut self, username: Username) -> Result<(), Error> {
        self.interaction(CreateUser { username }).await
    }

    pub async fn set_password(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), Error> {
        self.interaction(SetPassword { username, password }).await
    }

    pub async fn find_user(&mut self, username: Username) -> Result<Option<User>, Error> {
        self.interaction(FindUser { username }).await
    }

    pub async fn create_session(&mut self, user_id: Id, key: Key) -> Result<(), Error> {
        self.interaction(CreateSession { user_id, key }).await
    }

    pub async fn find_session(&mut self, key: Key) -> Result<Option<Session>, Error> {
        self.interaction(FindSession { key }).await
    }
}

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

struct FindUser {
    username: Username,
}

impl Interaction for FindUser {
    type Output = Option<User>;
}

pub struct CreateSession {
    user_id: Id,
    key: Key,
}

impl Interaction for CreateSession {
    type Output = ();
}

struct FindSession {
    key: Key,
}

impl Interaction for FindSession {
    type Output = Option<Session>;
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
        wait(|| self.dba().create_user(input.username)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<SetPassword> for DatabaseActor {
    async fn handle(&mut self, input: SetPassword) -> Result<(), Error> {
        // TODO: Protect password
        wait(|| self.dba().set_password(input.username, input.password)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<FindUser> for DatabaseActor {
    async fn handle(&mut self, input: FindUser) -> Result<Option<User>, Error> {
        wait(|| self.dba().find_user(input.username)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<CreateSession> for DatabaseActor {
    async fn handle(&mut self, input: CreateSession) -> Result<(), Error> {
        wait(|| self.dba().create_session(input.user_id, input.key)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<FindSession> for DatabaseActor {
    async fn handle(&mut self, input: FindSession) -> Result<Option<Session>, Error> {
        wait(|| self.dba().find_session(input.key)).map_err(Error::from)
    }
}

/// DatabaseActor routines.
impl DatabaseActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
