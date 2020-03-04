use async_trait::async_trait;
use failure::Error;
use futures::StreamExt;
use meio::{Actor, Context};
use std::net::SocketAddr;
use warp::{
    ws::{WebSocket, Ws},
    Filter, Reply,
};

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

#[async_trait]
impl Actor for Server {
    type Message = ();

    async fn routine(&mut self, _ctx: Context<Self>) -> Result<(), Error> {
        let routes = warp::path("live").and(warp::ws()).map(WsHandler::upgrade);
        warp::serve(routes).run(self.addr).await;
        Ok(())
    }
}

struct WsHandler {
    websocket: WebSocket,
}

impl WsHandler {
    fn upgrade(ws: Ws) -> impl Reply {
        ws.on_upgrade(Self::handle)
    }

    async fn handle(websocket: WebSocket) {
        let this = Self { websocket };
        if let Err(err) = this.routine().await {
            log::error!("WebSocket error: {}", err);
        }
    }

    async fn routine(self) -> Result<(), Error> {
        let (tx, rx) = self.websocket.split();
        rx.forward(tx).await?;
        Ok(())
    }
}
