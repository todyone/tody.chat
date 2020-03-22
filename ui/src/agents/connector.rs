use anyhow::Error;
use protocol::{ChannelUpdate, ClientToServer, Credentials, Key, LoginUpdate, ServerToClient};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use url::Url;
use yew::format::Json;
use yew::services::{
    storage::{Area, StorageService},
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
};
use yew::worker::*;

#[derive(Error, Debug)]
enum ConnectorError {
    #[error("can't get window object")]
    NoWindow,
    #[error("can't convert location to a string")]
    NoString,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum LoginStatus {
    Unauthorized,
    NeedCredentials { fail: Option<String> },
    LoggedIn,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Info {
    ConnectionInfo,
    LoginInfo,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    SetCredentials(Credentials),
    Subscribe(HashSet<Info>),
    CreateChannel(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Notification {
    ConnectionStatus(ConnectionStatus),
    LoginStatus(LoginStatus),
}

enum LoginBy {
    ByKey(Key),
    ByCredentials(Credentials),
}

/// Keeps connection to WebSockets automatically.
pub struct Connector {
    link: AgentLink<Self>,
    connection_status: ConnectionStatus,
    login_status: LoginStatus,
    service: WebSocketService,
    storage: StorageService,
    subscribers: HashSet<HandlerId>,
    ws: Option<WebSocketTask>,
    login_by: Option<LoginBy>,
}

#[derive(Debug)]
pub enum Msg {
    WsReady(Result<ServerToClient, Error>),
    WsStatus(WebSocketStatus),
}

impl Agent for Connector {
    type Reach = Context;
    type Message = Msg;
    type Input = Action;
    type Output = Notification;

    fn create(link: AgentLink<Self>) -> Self {
        // TODO: Implement this in yew:
        // link.send_message(Msg::Connect);
        let storage = StorageService::new(Area::Local).expect("Can't initialize LocalStorage");
        let mut this = Self {
            link,
            connection_status: ConnectionStatus::Disconnected,
            login_status: LoginStatus::Unauthorized,
            service: WebSocketService::new(),
            storage,
            subscribers: HashSet::new(),
            ws: None,
            login_by: None,
        };
        this.restore_key();
        this
    }

    fn update(&mut self, msg: Self::Message) {
        log::info!("Connector agent message: {:?}", msg);
        match msg {
            Msg::WsReady(res) => match res {
                Ok(ServerToClient::LoginUpdate(update)) => {
                    self.login_update(update);
                }
                Ok(ServerToClient::ChannelUpdate(update)) => {
                    self.channel_update(update);
                }
                Ok(ServerToClient::ChannelCreated(_)) => {}
                Ok(ServerToClient::Fail { reason: _ }) => {}
                Err(err) => {
                    log::error!("WS incoming error: {}", err);
                }
            },
            Msg::WsStatus(status) => match status {
                WebSocketStatus::Opened => {
                    self.set_connection_status(ConnectionStatus::Connected);
                    self.login();
                }
                WebSocketStatus::Closed | WebSocketStatus::Error => {
                    self.set_connection_status(ConnectionStatus::Disconnected);
                }
            },
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        log::trace!("Connector msg: {:?}", msg);
        match msg {
            Action::SetCredentials(creds) => {
                // Remove automatic login and wait for the new token
                self.remove_key();
                self.login_by = Some(LoginBy::ByCredentials(creds));
                self.login();
                // TODO: Set it on authorized
                //self.set_status(Status::LoggedIn);
            }
            Action::Subscribe(set) => {}
            Action::CreateChannel(channel_name) => {
                self.create_channel(channel_name);
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
        self.send_status_to(id);
        // Connect if first consumer appeared
        if !self.subscribers.is_empty() && self.ws.is_none() {
            if let Err(err) = self.connect() {
                log::error!("Can't connect to a server by WebSocket: {}", err);
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
        // Disconnect if there is no any listener remained
        if self.subscribers.is_empty() {
            self.ws.take();
        }
    }
}

impl Connector {
    fn login_update(&mut self, update: LoginUpdate) {
        match update {
            LoginUpdate::LoggedIn { key } => {
                let status = LoginStatus::LoggedIn;
                self.set_login_status(status);
                self.store_key(key);
            }
            LoginUpdate::LoginFail => {
                let reason;
                // Reset login_by field
                match self.login_by.take() {
                    Some(LoginBy::ByKey(_)) => {
                        reason = "Session expired".to_string();
                    }
                    Some(LoginBy::ByCredentials(_)) => {
                        reason = "Bad credentials".to_string();
                    }
                    None => {
                        unreachable!("Login failed without login info.");
                    }
                }
                let fail = Some(reason);
                let status = LoginStatus::NeedCredentials { fail };
                self.set_login_status(status);
            }
        }
    }

    fn channel_update(&mut self, update: ChannelUpdate) {
        todo!();
    }
}

impl Connector {
    const KEY: &'static str = "tody.chat.login_key";

    fn store_key(&mut self, key: Key) {
        self.storage.store(Self::KEY, Json(&key));
        self.login_by = Some(LoginBy::ByKey(key));
    }

    fn restore_key(&mut self) {
        let Json(key) = self.storage.restore(Self::KEY);
        self.login_by = key.ok().map(LoginBy::ByKey);
    }

    fn remove_key(&mut self) {
        self.storage.remove(Self::KEY);
    }
}

impl Connector {
    fn login(&mut self) {
        if let Some(login_by) = self.login_by.as_ref() {
            let msg = {
                match login_by {
                    LoginBy::ByCredentials(creds) => {
                        ClientToServer::CreateSession(creds.to_owned())
                    }
                    LoginBy::ByKey(key) => ClientToServer::RestoreSession(key.to_owned()),
                }
            };
            self.ws.as_mut().unwrap().send(Json(&msg));
        } else {
            let status = LoginStatus::NeedCredentials { fail: None };
            self.set_login_status(status);
        }
        // TODO: Schedule reconnection...
    }

    fn create_channel(&mut self, channel_name: String) {
        let msg = ClientToServer::CreateChannel(channel_name);
        self.ws.as_mut().unwrap().send(Json(&msg));
    }

    fn set_connection_status(&mut self, connection_status: ConnectionStatus) {
        self.connection_status = connection_status;
        let notification = Notification::ConnectionStatus(self.connection_status.clone());
        self.notify_subscribers(notification);
    }

    fn set_login_status(&mut self, login_status: LoginStatus) {
        self.login_status = login_status;
        let notification = Notification::LoginStatus(self.login_status.clone());
        self.notify_subscribers(notification);
    }

    fn connect(&mut self) -> Result<(), Error> {
        let mut url: Url = web_sys::window()
            .ok_or(ConnectorError::NoWindow)?
            .location()
            .to_string()
            .as_string()
            .ok_or(ConnectorError::NoString)?
            .parse()?;
        let scheme = if url.scheme().ends_with("s") {
            "wss"
        } else {
            "ws"
        };
        url.set_scheme(scheme);
        url.set_path("/live");
        let url = url.to_string();
        log::info!("Location: {}", url);
        let callback = self.link.callback(|Json(data)| Msg::WsReady(data));
        let notification = self.link.callback(Msg::WsStatus);
        let ws = self
            .service
            .connect(&url, callback, notification)
            .expect("Can't use WebSockets");
        self.ws = Some(ws);
        Ok(())
    }

    fn send_status_to(&self, id: HandlerId) {
        let connection_status = self.connection_status.clone();
        let notification = Notification::ConnectionStatus(connection_status);
        self.link.respond(id, notification);
    }

    fn notify_subscribers(&self, notification: Notification) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, notification.clone());
        }
    }
}
