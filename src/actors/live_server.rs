use crate::actors::Engine;
use crate::assets::{read_assets, Assets};
use anyhow::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use headers::{ContentType, HeaderMapExt};
use meio::{wrapper, Actor, Context};
use protocol::{ClientToServer, ServerToClient};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::block_in_place;
use warp::{
    http::{StatusCode, Uri},
    path::Tail,
    ws::{Message, WebSocket, Ws},
    Filter, Reply,
};

wrapper!(LiveServer for LiveServerActor);

impl LiveServer {
    pub fn start(addr: SocketAddr, engine: Engine) -> Self {
        let actor = LiveServerActor { addr, engine };
        meio::spawn(actor)
    }
}

pub struct LiveServerActor {
    addr: SocketAddr,
    engine: Engine,
}

#[async_trait]
impl Actor for LiveServerActor {
    type Interface = LiveServer;

    fn generic_name() -> &'static str {
        "LiveServer"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await?;
        Ok(())
    }
}

impl LiveServerActor {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        let asset_handler = AssetHandler::new().await?;
        let index = warp::path::end().map(|| warp::redirect(Uri::from_static("/index.html")));
        let engine = self.engine.clone();
        let live = warp::path("live")
            .and(warp::ws())
            .map(move |ws| LiveHandler::upgrade(ws, engine.clone()));
        let assets = warp::path::tail().map(move |tail| asset_handler.handle(tail));
        let routes = index.or(live).or(assets);
        warp::serve(routes).run(self.addr).await;
        Ok(())
    }
}

#[derive(Clone)]
struct AssetHandler {
    assets: Arc<Assets>,
}

impl AssetHandler {
    async fn new() -> Result<Self, Error> {
        let assets = block_in_place(read_assets)?;
        Ok(Self {
            assets: Arc::new(assets),
        })
    }

    fn handle(&self, tail: Tail) -> impl Reply {
        log::trace!("Assets request: {}", tail.as_str());
        let mime = mime_guess::from_path(tail.as_str()).first_or_octet_stream();
        let mut resp = self
            .assets
            .get(tail.as_str())
            .map(|data| data.clone().into_response())
            .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response());
        resp.headers_mut().typed_insert(ContentType::from(mime));
        resp
    }
}

/// WebSocket handler for `LiveServerActor`.
/// Session of user's connection.
struct LiveHandler {
    engine: Engine,
}

impl LiveHandler {
    fn upgrade(ws: Ws, engine: Engine) -> impl Reply {
        ws.on_upgrade(|weboscket| Self::handle(weboscket, engine))
    }

    async fn handle(websocket: WebSocket, engine: Engine) {
        let this = Self { engine };
        if let Err(err) = this.routine(websocket).await {
            log::warn!("LiveHandler error: {}", err);
        }
    }

    async fn routine(mut self, websocket: WebSocket) -> Result<(), Error> {
        log::trace!("Live WebSocket session started");
        let (mut tx, mut rx) = websocket.split();
        while let Some(msg) = rx.next().await.transpose()? {
            if msg.is_text() || msg.is_binary() {
                let request: ClientToServer = serde_json::from_slice(msg.as_bytes())?;
                log::trace!("Received: {:?}", request);
            /* TODO: Send
            let bytes = serde_json::to_vec(&response)?;
            let message = Message::binary(bytes);
            tx.send(message).await?;
            */
            } else if msg.is_ping() || msg.is_pong() {
            } else if msg.is_close() {
                break;
            } else {
                // Ignore Ping and Pong messages
                log::warn!("Unhandled WebSocket message: {:?}", msg);
            }
        }
        Ok(())
    }
}
