use anyhow::Error;
use protocol::ServerToClient;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use url::Url;
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::worker::*;

#[derive(Error, Debug)]
enum ConnectorError {
    #[error("can't get window object")]
    NoWindow,
    #[error("can't convert location to a string")]
    NoString,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Status {
    Disconnected,
    Connected,
    LoggedIn,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    SetCredentials,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Notification {
    StatusChanged(Status),
}

/// Keeps connection to WebSockets automatically.
pub struct Connector {
    status: Status,
    ws: WebSocketService,
    link: AgentLink<Connector>,
    subscribers: HashSet<HandlerId>,
    task: Option<WebSocketTask>,
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
        Self {
            status: Status::Disconnected,
            ws: WebSocketService::new(),
            link,
            subscribers: HashSet::new(),
            task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        log::info!("Agent message: {:?}", msg);
        match msg {
            Msg::WsReady(_res) => {}
            Msg::WsStatus(status) => {
                match status {
                    WebSocketStatus::Opened => {
                        self.set_status(Status::Connected);
                    }
                    WebSocketStatus::Closed | WebSocketStatus::Error => {
                        self.set_status(Status::Disconnected);
                        // TODO: Schedule reconnection...
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Action::SetCredentials => {}
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
        self.send_status_to(id);
        if !self.subscribers.is_empty() {
            if let Err(err) = self.connect() {
                log::error!("Can't connect to a server by WebSocket: {}", err);
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl Connector {
    fn set_status(&mut self, status: Status) {
        self.status = status;
        let notification = Notification::StatusChanged(self.status.clone());
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
        let task = self
            .ws
            .connect(&url, callback, notification)
            .expect("Can't use WebSockets");
        self.task = Some(task);
        Ok(())
    }

    fn send_status_to(&self, id: HandlerId) {
        let status = self.status.clone();
        let notification = Notification::StatusChanged(status);
        self.link.respond(id, notification);
    }

    fn notify_subscribers(&self, notification: Notification) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, notification.clone());
        }
    }
}
