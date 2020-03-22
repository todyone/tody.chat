use crate::actors::Engine;
use crate::assets::{read_assets, Assets};
use crate::db::types::UserId;
use anyhow::Error;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use headers::{ContentType, HeaderMapExt};
use meio::{wrapper, Actor, Context};
use protocol::{ClientToServer, LoginUpdate, ServerToClient};
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
    engine: Engine,
    user_id: Option<UserId>,
}

impl LiveHandler {
    fn upgrade(ws: Ws, engine: Engine) -> impl Reply {
        ws.on_upgrade(|weboscket| Self::handle(weboscket, engine))
    }

    async fn handle(websocket: WebSocket, engine: Engine) {
        let this = Self {
            engine,
            user_id: None,
        };
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
                let response = self.process_request(request).await.unwrap_or_else(|err| {
                    ServerToClient::Fail {
                        reason: err.to_string(),
                    }
                });
                let bytes = serde_json::to_vec(&response)?;
                let message = Message::binary(bytes);
                // TODO: Consider: track numbers instead of sequental processing
                tx.send(message).await?;
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

    async fn process_request(&mut self, request: ClientToServer) -> Result<ServerToClient, Error> {
        match request {
            ClientToServer::CreateSession(creds) => {
                let user_res = self.engine.find_user(creds.username).await?;
                match user_res {
                    Some(user) if user.password == creds.password => {
                        let key = self.engine.create_session(user.id).await?;
                        self.user_id = Some(user.id);
                        let update = LoginUpdate::LoggedIn { key };
                        Ok(ServerToClient::LoginUpdate(update))
                    }
                    Some(_) | None => {
                        // Don't share the reason
                        let update = LoginUpdate::LoginFail;
                        Ok(ServerToClient::LoginUpdate(update))
                    }
                }
            }
            ClientToServer::RestoreSession(key) => {
                let session_res = self.engine.find_session(key.clone()).await?;
                // TODO: Check properly (with protection)
                match session_res {
                    Some(session) if session.key == key => {
                        // TODO: Update session (last_visit field)
                        self.user_id = Some(session.user_id);
                        let update = LoginUpdate::LoggedIn { key };
                        Ok(ServerToClient::LoginUpdate(update))
                    }
                    Some(_) | None => {
                        // Don't share the reason
                        let update = LoginUpdate::LoginFail;
                        Ok(ServerToClient::LoginUpdate(update))
                    }
                }
            }
            ClientToServer::CreateChannel(channel_name) => {
                if let Some(user_id) = self.user_id {
                    self.engine
                        .create_channel(channel_name.clone(), user_id)
                        .await?;
                    Ok(ServerToClient::ChannelCreated(channel_name))
                } else {
                    let update = LoginUpdate::LoginFail;
                    Ok(ServerToClient::LoginUpdate(update))
                }
            }
        }
    }
}
