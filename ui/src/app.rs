use crate::agents::connector::{ConnectionStatus, Connector, LoginStatus, Notification};
use crate::components::{Chat, Login, Splash};
use yew::prelude::*;

enum Scene {
    Splash,
    Login { fail: Option<String> },
    Main,
}

pub struct App {
    scene: Scene,
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<Connector>>,
}

pub enum Msg {
    FromConnector(Notification),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = Connector::bridge(callback);
        Self {
            scene: Scene::Splash,
            link,
            connector,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FromConnector(notification) => match notification {
                Notification::ConnectionStatus(status) => match status {
                    ConnectionStatus::Disconnected => {}
                    ConnectionStatus::Connected => {}
                },
                Notification::LoginStatus(status) => match status {
                    LoginStatus::Unauthorized => {
                        self.scene = Scene::Splash;
                    }
                    LoginStatus::NeedCredentials { fail } => {
                        self.scene = Scene::Login { fail };
                    }
                    LoginStatus::LoggedIn => {
                        self.scene = Scene::Main;
                    }
                },
                Notification::ChannelStatus(_) => {}
            },
        }
        true
    }

    fn view(&self) -> Html {
        match self.scene {
            Scene::Splash => {
                html! {
                    <Splash />
                }
            }
            Scene::Login { ref fail } => {
                html! {
                    <Login fail=fail />
                }
            }
            Scene::Main => {
                html! {
                    <Chat />
                }
            }
        }
    }
}
