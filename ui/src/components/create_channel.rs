use crate::agents::connector::{Action, ConnectionStatus, Connector, LoginStatus, Notification};
use yew::prelude::*;

pub struct CreateChannel {
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<Connector>>,
}

pub enum Msg {
    FromConnector(Notification),
    CreateChannel,
}

impl Component for CreateChannel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = Connector::bridge(callback);
        Self { link, connector }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromConnector(notification) => {}
            Msg::CreateChannel => {
                let action = Action::CreateChannel("test-channel".into());
                self.connector.send(action);
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <label>{ "Channel name" }</label>
                <input placeholder="Channel name" />
                <button onclick=self.link.callback(|_| Msg::CreateChannel)>{ "Create" }</button>
            </div>
        }
    }
}
