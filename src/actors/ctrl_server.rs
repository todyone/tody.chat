use crate::control::{ClientToController, ControllerProtocol};
use crate::network::{wrap, NetworkConnection};
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, StreamExt};
use meio::{Actor, Context};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub struct CtrlServer {
    addr: SocketAddr,
}

impl CtrlServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

#[async_trait]
impl Actor for CtrlServer {
    type Message = ();

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await?;
        Ok(())
    }
}

impl CtrlServer {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        let mut listener = TcpListener::bind(&self.addr).await?;
        let mut incoming = listener.incoming().fuse();
        while let Some(stream) = incoming.next().await {}
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

    async fn routine(mut self) -> Result<(), Error> {
        while let Some(msg) = self.connection.next().await.transpose()? {
            log::trace!("Ctrl message: {:?}", msg);
            match msg {
                ClientToController::CreateUser { .. } => {}
                ClientToController::SetPassword { .. } => {}
            }
        }
        Ok(())
    }
}