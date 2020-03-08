use crate::network::{wrap, CodecError, NetworkConnection, ProtocolCodec};
use crate::types::{Password, Username};
use anyhow::Error;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::net::TcpStream;

pub type ClientProtocol = ProtocolCodec<ClientToController, ControllerToClient>;

pub type ControllerProtocol = ProtocolCodec<ControllerToClient, ClientToController>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientToController {
    CreateUser {
        username: Username,
    },
    SetPassword {
        username: Username,
        password: Password,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerToClient {
    UserCreated { username: Username },
    PasswordSet { username: Username },
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
}

pub struct Controller {
    connection: NetworkConnection<ClientProtocol>,
}

impl Controller {
    pub async fn connect(address: &str) -> Result<Self, ControllerError> {
        let stream = TcpStream::connect(address).await?;
        let connection = wrap(stream);
        Ok(Self { connection })
    }

    pub async fn create_user(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), ControllerError> {
        let expected = username.clone();
        let msg = ClientToController::CreateUser { username };
        self.connection.send(msg).await?;
        let resp = self.connection.next().await.transpose()?;
        match resp {
            Some(ControllerToClient::UserCreated { username }) if username == expected => Ok(()),
            Some(other) => Err(ControllerError::UnexpectedResponse(other)),
            None => Err(ControllerError::NoResponse),
        }
    }
}
