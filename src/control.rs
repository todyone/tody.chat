use crate::db::types::{ChannelName, Password, Username};
use crate::network::{wrap, CodecError, NetworkConnection, ProtocolCodec};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::timeout;

pub type ClientProtocol = ProtocolCodec<ClientToController, ControllerToClient>;

pub type ControllerProtocol = ProtocolCodec<ControllerToClient, ClientToController>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientToController {
    CreateUser {
        username: Username,
        password: Password,
    },
    UpdatePassword {
        username: Username,
        password: Password,
    },
    CreateChannel {
        channel: ChannelName,
        username: Username,
    },
    GetChannels,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerToClient {
    UserCreated { username: Username },
    PasswordUpdated { username: Username },
    ChannelCreated { channel: ChannelName },
    ChannelsList { channels: Vec<ChannelName> },
    Fail(String),
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(ControllerToClient),
    #[error("no response")]
    NoResponse,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("codec error {0}")]
    CodecError(#[from] CodecError),
    #[error("timeout {0}")]
    Timeout(#[from] tokio::time::Elapsed),
}

const WAIT_TIMEOUT_SEC: u64 = 10;

pub struct Controller {
    connection: NetworkConnection<ClientProtocol>,
}

impl Controller {
    pub async fn connect(address: &str) -> Result<Self, ControllerError> {
        let stream = TcpStream::connect(address).await?;
        let connection = wrap(stream);
        Ok(Self { connection })
    }

    async fn interact(
        &mut self,
        msg: ClientToController,
    ) -> Result<ControllerToClient, ControllerError> {
        self.connection.send(msg).await?;
        timeout(
            Duration::from_secs(WAIT_TIMEOUT_SEC),
            self.connection.next(),
        )
        .await?
        .transpose()?
        .ok_or(ControllerError::NoResponse)
    }

    pub async fn create_user(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), ControllerError> {
        let expected = username.clone();
        let msg = ClientToController::CreateUser { username, password };
        match self.interact(msg).await? {
            ControllerToClient::UserCreated { username } if username == expected => Ok(()),
            other => Err(ControllerError::UnexpectedResponse(other)),
        }
    }

    pub async fn update_password(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), ControllerError> {
        let expected = username.clone();
        let msg = ClientToController::UpdatePassword { username, password };
        match self.interact(msg).await? {
            ControllerToClient::PasswordUpdated { username } if username == expected => Ok(()),
            other => Err(ControllerError::UnexpectedResponse(other)),
        }
    }

    pub async fn create_channel(
        &mut self,
        channel: ChannelName,
        username: Username,
    ) -> Result<(), ControllerError> {
        let expected = channel.clone();
        let msg = ClientToController::CreateChannel { channel, username };
        match self.interact(msg).await? {
            ControllerToClient::ChannelCreated { channel } if channel == expected => Ok(()),
            other => Err(ControllerError::UnexpectedResponse(other)),
        }
    }

    pub async fn get_channels(&mut self) -> Result<Vec<ChannelName>, ControllerError> {
        let msg = ClientToController::GetChannels;
        match self.interact(msg).await? {
            ControllerToClient::ChannelsList { channels } => Ok(channels),
            other => Err(ControllerError::UnexpectedResponse(other)),
        }
    }
}
