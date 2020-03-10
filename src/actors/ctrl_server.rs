use crate::actors::DatabaseWrapper;
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
    database: DatabaseWrapper,
}

impl CtrlServer {
    pub fn new(addr: SocketAddr, database: DatabaseWrapper) -> Self {
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
            CtrlHandler::upgrade(stream, self.database.clone());
        }
        Ok(())
    }
}

struct CtrlHandler {
    connection: NetworkConnection<ControllerProtocol>,
    database: DatabaseWrapper,
}

impl CtrlHandler {
    fn upgrade(stream: TcpStream, database: DatabaseWrapper) {
        tokio::spawn(Self::handle(stream, database));
    }

    async fn handle(stream: TcpStream, database: DatabaseWrapper) {
        let connection = wrap(stream);
        let this = Self {
            connection,
            database,
        };
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
                    let result = self.database.create_user(username.clone()).await;
                    let response = {
                        match result {
                            Ok(_) => ControllerToClient::UserCreated { username },
                            Err(err) => {
                                log::error!("Can't create user: {}", err);
                                todo!();
                            }
                        }
                    };
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
