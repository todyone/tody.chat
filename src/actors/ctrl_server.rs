use crate::actors::Database;
use crate::control::{ClientToController, ControllerProtocol, ControllerToClient};
use crate::network::{wrap, NetworkConnection};
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, SinkExt, StreamExt};
use meio::{Actor, Address, Context, Wrapper};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub struct CtrlServer {
    addr: SocketAddr,
    database: Address<Database>,
}

impl CtrlServer {
    pub fn new(addr: SocketAddr, database: Address<Database>) -> Self {
        Self { addr, database }
    }
}

#[async_trait]
impl Actor for CtrlServer {
    type Message = ();
    type Interface = Wrapper<Self>;

    fn generic_name() -> &'static str {
        "CtrlServer"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        log::debug!("CtrlServer started");
        self.run(ctx).await?;
        Ok(())
    }
}

impl CtrlServer {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        let mut listener = TcpListener::bind(&self.addr).await?;
        let mut incoming = listener.incoming().fuse();
        while let Some(stream) = incoming.next().await.transpose()? {
            CtrlHandler::upgrade(stream);
        }
        Ok(())
    }
}

struct CtrlHandler {
    connection: NetworkConnection<ControllerProtocol>,
}

impl CtrlHandler {
    fn upgrade(stream: TcpStream) {
        tokio::spawn(Self::handle(stream));
    }

    async fn handle(stream: TcpStream) {
        let connection = wrap(stream);
        let this = Self { connection };
        if let Err(err) = this.routine().await {
            log::error!("CtrlHandler error: {}", err);
        }
    }

    async fn send(&mut self, response: ControllerToClient) -> Result<(), Error> {
        self.connection.send(response).await.map_err(Error::from)
    }

    async fn routine(mut self) -> Result<(), Error> {
        log::debug!("CtrlHandler started");
        while let Some(msg) = self.connection.next().await.transpose()? {
            log::trace!("Ctrl message: {:?}", msg);
            match msg {
                ClientToController::CreateUser { username } => {
                    log::debug!("User created: {}", username);
                    let response = ControllerToClient::UserCreated { username };
                    self.send(response).await?;
                }
                ClientToController::SetPassword { username, password } => {
                    log::debug!("Password updated: {}", username);
                    let response = ControllerToClient::PasswordSet { username };
                    self.send(response).await?;
                }
            }
        }
        Ok(())
    }
}
