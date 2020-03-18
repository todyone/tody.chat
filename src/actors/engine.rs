//! This module contains an `Actor` that interacts
//! with a database using `Dba` tools, but keeps
//! data consistency. For example it creates a channel
//! with a membership record for a specified user.
//! But all user-specific actions like finding ids for
//! a corresponding users performed by `ctrl` and `live`
//! actors.
//!
//! Also `EngineActor` has to notify other actors about
//! changes. It's a central point of all changes applied to
//! a database.
//!
//! In the future `EngineActor` will wrap accompanied requests
//! to transactions. That's another reason why is better to keep
//! complex requests with a single instance.

// TODO: Rewrite this module to fully async
// when SQLite crates will support that.

use crate::db::{Dba, Session, User};
use crate::generators::generate_key;
use crate::types::{Id, Password, Username};
use anyhow::Error;
use async_trait::async_trait;
use meio::{wrapper, Actor, Address, Interaction, InteractionHandler};
use protocol::Key;
use tokio::task::block_in_place as wait;

/// `Engine` provides business logic methods to manage data.
wrapper!(Engine for EngineActor);

impl Engine {
    pub fn start() -> Self {
        let actor = EngineActor { dba: None };
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
        self.interaction(UpdatePassword { username, password })
            .await
    }

    pub async fn find_user(&mut self, username: Username) -> Result<Option<User>, Error> {
        self.interaction(FindUser { username }).await
    }

    pub async fn create_session(&mut self, user_id: Id) -> Result<Key, Error> {
        let key = generate_key();
        // TODO: Protect key here
        self.interaction(CreateSession {
            user_id,
            key: key.clone(),
        })
        .await
        .map(|_| key)
    }

    pub async fn find_session(&mut self, key: Key) -> Result<Option<Session>, Error> {
        // TODO: Check key here
        self.interaction(FindSession { key }).await
    }
}

pub struct EngineActor {
    dba: Option<Dba>,
}

pub struct CreateUser {
    username: Username,
}

impl Interaction for CreateUser {
    type Output = ();
}

struct UpdatePassword {
    username: Username,
    password: Password,
}

impl Interaction for UpdatePassword {
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

pub struct CreateChannel {
    /// Name of the channel.
    name: String,
    /// `Id` of channel's creator.
    user_id: Id,
}

impl Interaction for CreateChannel {
    type Output = (); // TODO: Return channel info? At least channel Id.
}

pub struct AddMember {
    channel_id: Id,
    user_id: Id,
}

impl Interaction for AddMember {
    type Output = ();
}

pub struct RemoveMember {
    channel_id: Id,
    user_id: Id,
}

impl Interaction for RemoveMember {
    type Output = ();
}

#[async_trait]
impl Actor for EngineActor {
    type Interface = Engine;

    fn generic_name() -> &'static str {
        "Engine"
    }

    async fn initialize(&mut self, _address: Address<Self>) -> Result<(), Error> {
        let dba = Dba::open()?;
        self.dba = Some(dba);
        wait(|| self.dba().initialize())?;
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<CreateUser> for EngineActor {
    async fn handle(&mut self, input: CreateUser) -> Result<(), Error> {
        wait(|| self.dba().create_user(input.username)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<UpdatePassword> for EngineActor {
    async fn handle(&mut self, input: UpdatePassword) -> Result<(), Error> {
        // TODO: Protect password
        wait(|| self.dba().set_password(input.username, input.password)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<FindUser> for EngineActor {
    async fn handle(&mut self, input: FindUser) -> Result<Option<User>, Error> {
        wait(|| self.dba().find_user(input.username)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<CreateSession> for EngineActor {
    async fn handle(&mut self, input: CreateSession) -> Result<(), Error> {
        wait(|| self.dba().create_session(input.user_id, input.key)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<FindSession> for EngineActor {
    async fn handle(&mut self, input: FindSession) -> Result<Option<Session>, Error> {
        wait(|| self.dba().find_session(input.key)).map_err(Error::from)
    }
}

/// EngineActor routines.
impl EngineActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
