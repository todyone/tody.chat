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

use crate::db::types::{ChannelId, ChannelName, Password, UserId, Username};
use crate::db::{Channel, Dba, DbaError, Session, User};
use crate::generators::generate_key;
use anyhow::Error;
use async_trait::async_trait;
use meio::{wrapper, Actor, Address, Interaction, InteractionHandler};
use protocol::Key;
use rusqlite::Error as SqlError;
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

    pub async fn find_user(&mut self, username: Username) -> Result<Option<User>, Error> {
        self.interaction(FindUser { username }).await
    }

    pub async fn create_session(&mut self, user_id: UserId) -> Result<Key, Error> {
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

    pub async fn create_channel(
        &mut self,
        channel: ChannelName,
        user_id: UserId,
    ) -> Result<(), Error> {
        self.interaction(CreateChannel { channel, user_id }).await
    }

    pub async fn get_channels(&mut self) -> Result<Vec<Channel>, Error> {
        self.interaction(GetChannels {}).await
    }

    pub async fn delete_channel(&mut self, channel: ChannelName) -> Result<(), Error> {
        self.interaction(DeleteChannel { channel }).await
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
    type Output = Option<User>;
}

#[derive(Debug)]
pub struct CreateSession {
    user_id: UserId,
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
    type Output = Option<Session>;
}

#[derive(Debug)]
pub struct CreateChannel {
    channel: ChannelName,
    /// Channel's creator (first member/owner)
    user_id: UserId,
}

impl Interaction for CreateChannel {
    type Output = (); // TODO: Return channel info? At least channel Id.
}

#[derive(Debug)]
pub struct GetChannels {
    // TODO: Add user's filter
}

impl Interaction for GetChannels {
    type Output = Vec<Channel>;
}

#[derive(Debug)]
pub struct AddMember {
    channel_id: ChannelId,
    user_id: UserId,
}

impl Interaction for AddMember {
    type Output = ();
}

#[derive(Debug)]
pub struct RemoveMember {
    channel_id: ChannelId,
    user_id: UserId,
}

impl Interaction for RemoveMember {
    type Output = ();
}

#[derive(Debug)]
pub struct DeleteChannel {
    channel: ChannelName,
}

impl Interaction for DeleteChannel {
    type Output = ();
}

#[async_trait]
impl Actor for EngineActor {
    type Interface = Engine;

    fn generic_name() -> &'static str {
        "Engine"
    }

    async fn initialize(&mut self, _address: Address<Self>) -> Result<(), Error> {
        std::fs::create_dir_all(crate::db::DATA_DIR)?;
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
    async fn handle(&mut self, input: FindUser) -> Result<Option<User>, Error> {
        optional(wait(|| self.dba().get_user(input.username)))
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
        optional(wait(|| self.dba().get_session(input.key)))
    }
}

#[async_trait]
impl InteractionHandler<DeleteChannel> for EngineActor {
    async fn handle(&mut self, input: DeleteChannel) -> Result<(), Error> {
        wait(|| self.dba().delete_channel(input.channel)).map_err(Error::from)
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

#[async_trait]
impl InteractionHandler<GetChannels> for EngineActor {
    async fn handle(&mut self, _: GetChannels) -> Result<Vec<Channel>, Error> {
        wait(|| self.dba().get_channels()).map_err(Error::from)
    }
}

fn optional<T>(res: Result<T, DbaError>) -> Result<Option<T>, Error> {
    match res {
        Ok(value) => Ok(Some(value)),
        Err(DbaError::DbError(SqlError::QueryReturnedNoRows)) => Ok(None),
        Err(err) => Err(Error::from(err)),
    }
}

/// EngineActor routines.
impl EngineActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
