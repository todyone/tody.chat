use crate::network::{CodecError, FramedConnection};
use crate::protocol::ClientProtocol;
use thiserror::Error;
use tokio::net::TcpStream;

#[derive(Error, Debug)]
pub enum ControllerError {
    /*
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(ControllerToClient),
    */
    #[error("no response")]
    NoResponse,
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("codec error {0}")]
    CodecError(#[from] CodecError),
    #[error("timeout {0}")]
    Timeout(#[from] tokio::time::Elapsed),
}

pub struct Controller {
    connection: FramedConnection<ClientProtocol>,
}

impl Controller {
    pub async fn connect(address: &str) -> Result<Self, ControllerError> {
        let stream = TcpStream::connect(address).await?;
        let connection = FramedConnection::wrap(stream);
        Ok(Self { connection })
    }
}
