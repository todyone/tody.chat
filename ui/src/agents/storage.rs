use protocol::Key;
use serde::{Deserialize, Serialize};
use yew::format::Json;
use yew::services::storage::{Area, StorageService};
use yew::worker::*;

pub struct Storage {
    link: AgentLink<Self>,
    storage: StorageService,
}

#[derive(Debug)]
pub enum Msg {}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    GetLoginKey,
    SetLoginKey(Key),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Notification {
    LoginKeyResult(Option<Key>),
}

impl Agent for Storage {
    type Reach = Context;
    type Message = Msg;
    type Input = Action;
    type Output = Notification;

    fn create(link: AgentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("Can't initialize LocalStorage");
        Self { link, storage }
    }

    fn update(&mut self, msg: Self::Message) {
        log::info!("Storage agent message: {:?}", msg);
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        log::trace!("Storage msg: {:?}", msg);
        match msg {
            Action::GetLoginKey => {
                let key = self.restore_key();
                let notification = Notification::LoginKeyResult(key);
                self.link.respond(id, notification);
            }
            Action::SetLoginKey(key) => {
                self.store_key(&key);
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {}

    fn disconnected(&mut self, id: HandlerId) {}
}

impl Storage {
    const KEY: &'static str = "tody.chat.login_key";

    fn store_key(&mut self, key: &Key) {
        self.storage.store(Self::KEY, Json(key));
    }

    fn restore_key(&mut self) -> Option<Key> {
        let Json(key) = self.storage.restore(Self::KEY);
        key.ok()
    }
}
