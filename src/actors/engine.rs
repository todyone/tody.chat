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

use crate::db::{Dba, DbaError, Session, User};
use crate::generators::generate_key;
use crate::types::{ChannelName, Id, Password, Username};
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

    pub async fn create_user(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), Error> {
        self.interaction(CreateUser { username, password }).await
    }

    pub async fn set_password(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), Error> {
        self.interaction(UpdatePassword { username, password })
            .await
    }

    pub async fn get_user(&mut self, username: Username) -> Result<User, Error> {
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

    pub async fn get_session(&mut self, key: Key) -> Result<Session, Error> {
        // TODO: Check key here
        self.interaction(FindSession { key }).await
    }

    pub async fn create_channel(&mut self, channel: ChannelName, user_id: Id) -> Result<(), Error> {
        self.interaction(CreateChannel { channel, user_id }).await
    }
}

pub struct EngineActor {
    dba: Option<Dba>,
}

#[derive(Debug)]
pub struct CreateUser {
    username: Username,
    password: Password,
}

impl Interaction for CreateUser {
    type Output = ();
}

#[derive(Debug)]
struct UpdatePassword {
    username: Username,
    password: Password,
}

impl Interaction for UpdatePassword {
    type Output = ();
}

#[derive(Debug)]
struct FindUser {
    username: Username,
}

impl Interaction for FindUser {
    type Output = User;
}

#[derive(Debug)]
pub struct CreateSession {
    user_id: Id,
    key: Key,
}

impl Interaction for CreateSession {
    type Output = ();
}

#[derive(Debug)]
struct FindSession {
    key: Key,
}

impl Interaction for FindSession {
    type Output = Session;
}

#[derive(Debug)]
pub struct CreateChannel {
    /// Name of the channel.
    channel: String,
    /// `Id` of channel's creator.
    user_id: Id,
}

impl Interaction for CreateChannel {
    type Output = (); // TODO: Return channel info? At least channel Id.
}

#[derive(Debug)]
pub struct AddMember {
    channel_id: Id,
    user_id: Id,
}

impl Interaction for AddMember {
    type Output = ();
}

#[derive(Debug)]
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
        wait(|| -> Result<(), DbaError> {
            log::trace!("Creating user: {:?}", input);
            // TODO: User RETURNING id possible?
            self.dba().create_user(input.username.clone())?;
            let user = self.dba().get_user(input.username)?;
            self.dba().set_password(user.id, input.password)?;
            Ok(())
        })
        .map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<UpdatePassword> for EngineActor {
    async fn handle(&mut self, input: UpdatePassword) -> Result<(), Error> {
        // TODO: Protect password
        wait(|| -> Result<(), DbaError> {
            log::trace!("Updating password: {:?}", input);
            let user = self.dba().get_user(input.username)?;
            self.dba().set_password(user.id, input.password)?;
            Ok(())
        })
        .map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<FindUser> for EngineActor {
    async fn handle(&mut self, input: FindUser) -> Result<User, Error> {
        wait(|| self.dba().get_user(input.username)).map_err(Error::from)
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
    async fn handle(&mut self, input: FindSession) -> Result<Session, Error> {
        wait(|| self.dba().get_session(input.key)).map_err(Error::from)
    }
}

#[async_trait]
impl InteractionHandler<CreateChannel> for EngineActor {
    async fn handle(&mut self, input: CreateChannel) -> Result<(), Error> {
        // TODO: Use TRANSACTION here
        wait(|| {
            log::trace!("Creating channel: {:?}", input);
            self.dba().create_channel(input.channel.clone())?;
            let channel = self.dba().get_channel(input.channel)?;
            self.dba().add_member(channel.id, input.user_id)?;
            Ok(())
        })
    }
}

/// EngineActor routines.
impl EngineActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
