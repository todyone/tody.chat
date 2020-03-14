use crate::actors::Database;
use crate::assets::{read_assets, Assets};
use crate::types::Id;
use anyhow::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use headers::{ContentType, HeaderMapExt};
use meio::{wrapper, Actor, Context};
use protocol::{ClientToServer, ServerToClient};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::block_in_place as wait;
use warp::{
    http::{StatusCode, Uri},
    path::Tail,
    ws::{Message, WebSocket, Ws},
    Filter, Reply,
};

wrapper!(LiveServer for LiveServerActor);

impl LiveServer {
    pub fn start(addr: SocketAddr, db: Database) -> Self {
        let actor = LiveServerActor { addr, db };
        meio::spawn(actor)
    }
}

pub struct LiveServerActor {
    addr: SocketAddr,
    db: Database,
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

/// WebSocket handler for `LiveServerActor`.
struct LiveHandler {
    db: Database,
    user_id: Option<Id>,
}

impl LiveHandler {
    fn upgrade(ws: Ws, db: Database) -> impl Reply {
        ws.on_upgrade(|weboscket| Self::handle(weboscket, db))
    }

    async fn handle(websocket: WebSocket, db: Database) {
        let this = Self { db, user_id: None };
        if let Err(err) = this.routine(websocket).await {
            log::error!("LiveHandler error: {}", err);
        }
    }

    async fn routine(mut self, websocket: WebSocket) -> Result<(), Error> {
        let (mut tx, mut rx) = websocket.split();
        while let Some(msg) = rx.next().await.transpose()? {
            let request: ClientToServer = serde_json::from_slice(msg.as_bytes())?;
            log::trace!("Received: {:?}", request);
            let response = self
                .process_request(request)
                .await
                .unwrap_or_else(|err| ServerToClient::Fail(err.to_string()));
            let bytes = serde_json::to_vec(&response)?;
            let message = Message::binary(bytes);
            // TODO: Consider: track numbers instead of sequental processing
            tx.send(message).await?;
        }
        Ok(())
    }

    async fn process_request(&mut self, request: ClientToServer) -> Result<ServerToClient, Error> {
        match request {
            ClientToServer::Login(creds) => {
                let user = self.db.find_user(creds.username).await?;
                match user {
                    Some(user) if user.password == creds.password => Ok(ServerToClient::LoggedIn),
                    Some(_) | None => {
                        // Don't share the reason
                        Ok(ServerToClient::LoginFail)
                    }
                }
            }
        }
    }
}
