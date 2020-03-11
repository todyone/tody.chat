use crate::actors::Database;
use crate::assets::{read_assets, Assets};
use crate::types::Id;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use headers::{ContentType, HeaderMapExt};
use meio::{Actor, Context, Wrapper};
use protocol::ClientToServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::block_in_place as wait;
use warp::{
    http::{StatusCode, Uri},
    path::Tail,
    ws::{WebSocket, Ws},
    Filter, Reply,
};

pub struct LiveServer {
    addr: SocketAddr,
    db: Database,
}

impl LiveServer {
    pub fn new(addr: SocketAddr, db: Database) -> Self {
        Self { addr, db }
    }
}

#[async_trait]
impl Actor for LiveServer {
    type Message = ();
    type Interface = Wrapper<Self>;

    fn generic_name() -> &'static str {
        "LiveServer"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await?;
        Ok(())
    }
}

impl LiveServer {
    async fn run(&mut self, _: Context<Self>) -> Result<(), Error> {
        let asset_handler = AssetHandler::new().await?;
        let index = warp::path::end().map(|| warp::redirect(Uri::from_static("/index.html")));
        let db = self.db.clone();
        let live = warp::path("live")
            .and(warp::ws())
            .map(move |ws| LiveHandler::upgrade(ws, db.clone()));
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
        let assets = wait(read_assets)?;
        Ok(Self {
            assets: Arc::new(assets),
        })
    }

    fn handle(&self, tail: Tail) -> impl Reply {
        log::trace!("req: {}", tail.as_str());
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

/// WebSocket handler for `LiveServer`.
struct LiveHandler {
    websocket: WebSocket,
    db: Database,
    user_id: Option<Id>,
}

impl LiveHandler {
    fn upgrade(ws: Ws, db: Database) -> impl Reply {
        ws.on_upgrade(|weboscket| Self::handle(weboscket, db))
    }

    async fn handle(websocket: WebSocket, db: Database) {
        let this = Self {
            websocket,
            db,
            user_id: None,
        };
        if let Err(err) = this.routine().await {
            log::error!("LiveHandler error: {}", err);
        }
    }

    async fn routine(self) -> Result<(), Error> {
        let (_tx, mut rx) = self.websocket.split();
        while let Some(msg) = rx.next().await.transpose()? {
            let request: ClientToServer = serde_json::from_slice(msg.as_bytes())?;
            log::trace!("Received: {:?}", request);
            match request {
                ClientToServer::Login(creds) => {
                    // TODO: 1. Get user
                    // TODO: 2. Check password
                }
            }
        }
        Ok(())
    }
}
