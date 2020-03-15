use crate::agents::{connector, storage};
use protocol::Key;
use yew::prelude::*;

pub struct Splash {
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<connector::Connector>>,
    storage: Box<dyn Bridge<storage::Storage>>,
}

pub enum Msg {
    FromConnector(connector::Notification),
    FromStorage(storage::Notification),
}

impl Component for Splash {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = connector::Connector::bridge(callback);
        let callback = link.callback(|n| Msg::FromStorage(n));
        let storage = storage::Storage::bridge(callback);
        let mut this = Self {
            link,
            connector,
            storage,
        };
        this.request_login_key();
        this
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromConnector(msg) => {}
            Msg::FromStorage(msg) => match msg {
                storage::Notification::LoginKeyResult(opt_key) => {
                    if let Some(key) = opt_key {
                        self.login_with_key(key);
                    }
                }
            },
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="loader">
                <img src="/loader.svg" />
            </div>
        }
    }
}

impl Splash {
    fn request_login_key(&mut self) {
        let action = storage::Action::GetLoginKey;
        self.storage.send(action);
    }

    fn login_with_key(&mut self, key: Key) {
        let action = connector::Action::SetKey(key);
        self.connector.send(action);
    }
}
