use crate::agents::connector;
use crate::components::{Chat, Login, Splash};
use yew::prelude::*;

enum Scene {
    Splash,
    Login,
    Main,
}

pub struct App {
    scene: Scene,
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<connector::Connector>>,
}

pub enum Msg {
    Login,
    FromConnector(connector::Notification),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::FromConnector(n));
        let connector = connector::Connector::bridge(callback);
        Self {
            scene: Scene::Splash,
            link,
            connector,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Login => {
                self.scene = Scene::Main;
            }
            Msg::FromConnector(notification) => match notification {
                connector::Notification::StatusChanged(status) => match status {
                    connector::Status::LoggedIn => {
                        self.scene = Scene::Main;
                    }
                    connector::Status::Disconnected => {
                        self.scene = Scene::Splash;
                    }
                    connector::Status::Connected => {
                        self.scene = Scene::Login;
                    }
                },
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
            Scene::Login => {
                html! {
                    <Login />
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
