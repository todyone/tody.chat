use crate::components::{Chat, Login};
use yew::prelude::*;

enum Scene {
    Login,
    Main,
}

pub struct App {
    link: ComponentLink<Self>,
    scene: Scene,
}

pub enum Msg {
    Login,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            scene: Scene::Login,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Login => {
                self.scene = Scene::Main;
            }
        }
        true
    }

    fn view(&self) -> Html {
        match self.scene {
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
