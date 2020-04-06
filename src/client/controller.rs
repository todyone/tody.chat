use crate::network::ClientConnection;
use futures::{SinkExt, StreamExt};
use protocol::{
    request::{
        actions::{Info, InfoType},
        ClientToServer,
    },
    response::ServerToClient,
};
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum Error {
    /*
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(ControllerToClient),
    */
    #[error("no response")]
    NoResponse,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("codec error {0}")]
    CodecError(#[from] crate::network::CodecError),
    #[error("timeout {0}")]
    Timeout(#[from] tokio::time::Elapsed),
}

const WAIT_TIMEOUT_SEC: u64 = 10;

pub struct Controller {
    connection: ClientConnection,
}

impl Controller {
    pub async fn connect(address: &str) -> Result<Self, Error> {
        let stream = TcpStream::connect(address).await?;
        let connection = ClientConnection::wrap(stream);
        Ok(Self { connection })
    }

    pub async fn send(&mut self, msg: impl Into<ClientToServer>) -> Result<(), Error> {
        self.connection.send(msg.into()).await.map_err(Error::from)
    }

    pub async fn recv(&mut self) -> Result<ServerToClient, Error> {
        timeout(
            Duration::from_secs(WAIT_TIMEOUT_SEC),
            self.connection.next(),
        )
        .await?
        .transpose()?
        .ok_or(Error::NoResponse)
    }

    pub async fn subscribe(&mut self, info_type: InfoType) -> Result<(), Error> {
        let info = Info::Subscribe(info_type);
        self.send(info).await
    }

    pub async fn unsubscribe(&mut self, info_type: InfoType) -> Result<(), Error> {
        let info = Info::Unsubscribe(info_type);
        self.send(info).await
    }
}
