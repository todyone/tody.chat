use crate::agents::{connector, storage};
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
        Self {
            link,
            connector,
            storage,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
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
