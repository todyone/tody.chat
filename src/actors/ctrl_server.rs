use crate::actors::Engine;
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
    pub fn start(addr: SocketAddr, engine: Engine) -> Self {
        let actor = CtrlServerActor { addr, engine };
        meio::spawn(actor)
    }
}

pub struct CtrlServerActor {
    addr: SocketAddr,
    engine: Engine,
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
            CtrlHandler::upgrade(stream, self.engine.clone());
        }
        Ok(())
    }
}

struct CtrlHandler {
    connection: NetworkConnection<ControllerProtocol>,
    engine: Engine,
}

impl CtrlHandler {
    fn upgrade(stream: TcpStream, engine: Engine) {
        tokio::spawn(Self::handle(stream, engine));
    }

    async fn handle(stream: TcpStream, engine: Engine) {
        let connection = wrap(stream);
        let this = Self { connection, engine };
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
                    log::debug!("Creating user: {}", username);
                    let response = self
                        .engine
                        .create_user(username.clone())
                        .await
                        .map(|_| ControllerToClient::UserCreated { username })
                        .unwrap_or_else(|err| {
                            log::error!("Can't create user: {}", err);
                            ControllerToClient::Fail(err.to_string())
                        });
                    self.send(response).await?;
                }
                ClientToController::SetPassword { username, password } => {
                    log::debug!("Updating password: {}", username);
                    let response = self
                        .engine
                        .set_password(username.clone(), password)
                        .await
                        .map(|_| ControllerToClient::PasswordSet { username })
                        .unwrap_or_else(|err| {
                            log::error!("Can't set new password: {}", err);
                            ControllerToClient::Fail(err.to_string())
                        });
                    self.send(response).await?;
                }
                ClientToController::CreateChannel { channel, username } => {
                    let user = self.engine.find_user(username.clone()).await;
                    // TODO: Refactor that match part
                    let response = {
                        match user {
                            Ok(Some(user)) => {
                                log::debug!("Creating channel: {}", channel);
                                let response = self
                                    .engine
                                    .create_channel(channel.clone(), user.id)
                                    .await
                                    .map(|_| ControllerToClient::ChannelCreated { channel })
                                    .unwrap_or_else(|err| {
                                        log::error!("Can't create a channel: {}", err);
                                        ControllerToClient::Fail(err.to_string())
                                    });
                                response
                            }
                            Ok(None) => {
                                log::error!("Can't find user: {}", username);
                                ControllerToClient::Fail("user doesn't exists".into())
                            }
                            Err(err) => {
                                log::error!("Can't find user: {}", err);
                                ControllerToClient::Fail(err.to_string())
                            }
                        }
                    };
                    self.send(response).await?;
                }
            }
        }
        Ok(())
    }
}
