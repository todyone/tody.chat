use crate::agents::connector::{
    Action, ChannelStatus, ConnectionStatus, Connector, Info, LoginStatus, Notification,
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub oncomplete: Callback<()>,
}

pub struct CreateChannel {
    link: ComponentLink<Self>,
    channel_name: String,
    connector: Box<dyn Bridge<Connector>>,
    oncomplete: Callback<()>,
}

pub enum Msg {
    FromConnector(Notification),
    UpdateChannelName(String),
    CreateChannel,
}

impl Component for CreateChannel {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = Connector::bridge(callback);
        let mut this = Self {
            link,
            channel_name: String::new(),
            connector,
            oncomplete: props.oncomplete,
        };
        this.subscribe();
        this
    }

    // TODO: I feel that we sould add `init(&mut self)` method to Yew::Component?

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromConnector(notification) => {
                log::debug!("Notification: {:?}", notification);
                match notification {
                    Notification::ChannelStatus(ChannelStatus::ChannelCreated(channel_name)) => {
                        if self.channel_name == channel_name {
                            self.oncomplete.emit(());
                        }
                    }
                    _ => {}
                }
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.oncomplete = props.oncomplete;
        true
    }
}

impl CreateChannel {
    fn subscribe(&mut self) {
        let info_set = vec![Info::ChannelInfo].into_iter().collect();
        self.connector.send(Action::Subscribe(info_set));
    }
}
