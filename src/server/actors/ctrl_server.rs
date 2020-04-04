use crate::network::ServerConnection;
use crate::server::actors::Engine;
use anyhow::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use meio::{wrapper, Actor, Context};
use protocol::{request::ClientToServer, response::ServerToClient};
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
    connection: ServerConnection,
    engine: Engine,
}

impl CtrlHandler {
    fn upgrade(stream: TcpStream, engine: Engine) {
        tokio::spawn(Self::handle(stream, engine));
    }

    async fn handle(stream: TcpStream, engine: Engine) {
        let connection = ServerConnection::wrap(stream);
        let this = Self { connection, engine };
        if let Err(err) = this.routine().await {
            log::error!("CtrlHandler error: {}", err);
        }
    }

    async fn send(&mut self, response: ServerToClient) -> Result<(), Error> {
        self.connection.send(response).await.map_err(Error::from)
    }

    async fn routine(mut self) -> Result<(), Error> {
        log::debug!("CtrlHandler started");
        while let Some(msg) = self.connection.next().await.transpose()? {
            log::trace!("Ctrl message: {:?}", msg);
        }
        Ok(())
    }
}
