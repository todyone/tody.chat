use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::worker::*;

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
    link: AgentLink<Connector>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for Connector {
    type Reach = Context;
    type Message = ();
    type Input = Action;
    type Output = Notification;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            status: Status::Disconnected,
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Action::SetCredentials => {}
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
        self.send_status_to(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl Connector {
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
