use crate::agents::{Connector, Notification};
use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<Connector>>,
}

pub enum Msg {
    SendCredentials,
    Notification(Notification),
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::Notification(n));
        let connector = Connector::bridge(callback);
        Self { link, connector }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div><p onclick=self.link.callback(|_| Msg::SendCredentials)>{ "Login" }</p></div>
        }
    }
}
