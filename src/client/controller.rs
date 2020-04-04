use crate::network::ClientConnection;
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
    CodecError(#[from] crate::network::CodecError),
    #[error("timeout {0}")]
    Timeout(#[from] tokio::time::Elapsed),
}

pub struct Controller {
    connection: ClientConnection,
}

impl Controller {
    pub async fn connect(address: &str) -> Result<Self, ControllerError> {
        let stream = TcpStream::connect(address).await?;
        let connection = ClientConnection::wrap(stream);
        Ok(Self { connection })
    }
}
