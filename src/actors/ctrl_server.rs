use crate::actors::Database;
use crate::control::{ClientToController, ControllerProtocol, ControllerToClient};
use crate::network::{wrap, NetworkConnection};
use anyhow::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use meio::{wrapper, Actor, Context};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

wrapper!(CtrlServer for CtrlServerActor);

impl CtrlServer {
    pub fn start(addr: SocketAddr, db: Database) -> Self {
        let actor = CtrlServerActor { addr, db };
        meio::spawn(actor)
    }
}

pub struct CtrlServerActor {
    addr: SocketAddr,
    db: Database,
}

#[async_trait]
impl Actor for CtrlServerActor {
    type Interface = CtrlServer;

    fn generic_name() -> &'static str {
        "CtrlServer"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        log::debug!("CtrlServer started");
        self.run(ctx).await?;
        Ok(())
    }
}

impl CtrlServerActor {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        let mut listener = TcpListener::bind(&self.addr).await?;
        let mut incoming = listener.incoming().fuse();
        while let Some(stream) = incoming.next().await.transpose()? {
            CtrlHandler::upgrade(stream, self.db.clone());
        }
        Ok(())
    }
}

struct CtrlHandler {
    connection: NetworkConnection<ControllerProtocol>,
    db: Database,
}

impl CtrlHandler {
    fn upgrade(stream: TcpStream, db: Database) {
        tokio::spawn(Self::handle(stream, db));
    }

    async fn handle(stream: TcpStream, db: Database) {
        let connection = wrap(stream);
        let this = Self { connection, db };
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
                    let result = self.db.create_user(username.clone()).await;
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
