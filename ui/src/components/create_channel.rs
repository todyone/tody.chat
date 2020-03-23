use crate::agents::connector::{Action, ConnectionStatus, Connector, LoginStatus, Notification};
use yew::prelude::*;

pub struct CreateChannel {
    link: ComponentLink<Self>,
    channel_name: String,
    connector: Box<dyn Bridge<Connector>>,
}

pub enum Msg {
    FromConnector(Notification),
    UpdateChannelName(String),
    CreateChannel,
}

impl Component for CreateChannel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = Connector::bridge(callback);
        Self {
            link,
            channel_name: String::new(),
            connector,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromConnector(notification) => {
                log::debug!("Notification: {:?}", notification);
            }
            Msg::UpdateChannelName(channel_name) => {
                self.channel_name = channel_name;
            }
            Msg::CreateChannel => {
                let action = Action::CreateChannel(self.channel_name.clone());
                self.connector.send(action);
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <label>{ "Channel name" }</label>
                <input value=self.channel_name
                       placeholder="Channel"
                       oninput=self.link.callback(|e: InputData| Msg::UpdateChannelName(e.value)) />
                <button onclick=self.link.callback(|_| Msg::CreateChannel)>{ "Create" }</button>
            </div>
        }
    }
}
