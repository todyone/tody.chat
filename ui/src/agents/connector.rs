use anyhow::Error;
use protocol::{
    ChannelUpdate, ClientToServer, Credentials, Delta, Key, LoginUpdate, Reaction, ServerToClient,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelStatus {
    ChannelCreated(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Info {
    ConnectionInfo,
    LoginInfo,
    ChannelInfo,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Subscribe(HashSet<Info>),

    // TODO: Remove duplicatied requests
    SetCredentials(Credentials),

    // TODO: Remove duplicatied requests
    CreateChannel(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Response {
    Reaction(Reaction),
    Notification(Notification),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Notification {
    ConnectionStatus(ConnectionStatus),
    LoginStatus(LoginStatus),
    ChannelStatus(ChannelStatus),
}

enum LoginBy {
    ByKey(Key),
    ByCredentials(Credentials),
}

struct Task {
    recipient: HandlerId,
    request: Option<ClientToServer>,
}

/// Keeps connection to WebSockets automatically.
pub struct Connector {
    link: AgentLink<Self>,
    connection_status: ConnectionStatus,
    login_status: LoginStatus,
    service: WebSocketService,
    storage: StorageService,
    subscribers: HashSet<HandlerId>,
    subscriptions: HashMap<Info, HashSet<HandlerId>>,
    ws: Option<WebSocketTask>,
    login_by: Option<LoginBy>,
    task_queue: VecDeque<Task>,
    active_task: Option<Task>,
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
            subscriptions: HashMap::new(),
            ws: None,
            login_by: None,
            task_queue: VecDeque::new(),
            active_task: None,
        };
        this.restore_key();
        this
    }

    fn update(&mut self, msg: Self::Message) {
        log::info!("Connector agent message: {:?}", msg);
        match msg {
            Msg::WsReady(res) => match res {
                Ok(ServerToClient::Delta(delta)) => {
                    match delta {
                        Delta::LoginUpdate(update) => {
                            self.login_update(update);
                        }
                        Delta::ChannelUpdate(update) => {
                            self.channel_update(update);
                        } /* TODO: Track results and notify about tasks
                          Delta::ChannelCreated(channel_name) => {
                              let msg =
                                  Notification::ChannelStatus(ChannelStatus::ChannelCreated(channel_name));
                              self.notify_subscribers(Info::ChannelInfo, msg);
                          }
                          */
                    }
                }
                Ok(ServerToClient::Reaction(reaction)) => {
                    // Drop active task on any reaction
                    if let Some(task) = self.active_task.take() {
                        // TODO: self.send_reaction_to(task.recipient, reaction);
                    }
                }
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

    fn handle_input(&mut self, msg: Self::Input, handler: HandlerId) {
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
            Action::Subscribe(set) => {
                for info in set {
                    self.subscriptions.entry(info).or_default().insert(handler);
                }
                // TODO: Check and track difference
            }
            Action::CreateChannel(channel_name) => {
                self.create_channel(channel_name);
            }
        }
    }

    /// Consumer connected
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

    /// Consumer disconnected
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
        // Disconnect if there is no any listener remained
        if self.subscribers.is_empty() {
            self.ws.take();
        }
    }
}

impl Connector {
    fn process_task(&mut self) {
        let connected = matches!(self.connection_status, ConnectionStatus::Connected);
        if connected && self.active_task.is_none() {
            if let Some(mut task) = self.task_queue.pop_front() {
                let msg = task.request.take().expect("empty task in a queue");
                self.active_task = Some(task);
                self.ws.as_mut().unwrap().send(Json(&msg));
            }
        }
    }

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
        self.notify_all_subscribers(notification);
    }

    fn set_login_status(&mut self, login_status: LoginStatus) {
        self.login_status = login_status;
        let notification = Notification::LoginStatus(self.login_status.clone());
        self.notify_all_subscribers(notification);
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

    fn notify_subscribers(&self, info: Info, notification: Notification) {
        if let Some(set) = self.subscriptions.get(&info) {
            for sub in set {
                self.link.respond(*sub, notification.clone());
            }
        }
    }

    fn notify_all_subscribers(&self, notification: Notification) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, notification.clone());
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
