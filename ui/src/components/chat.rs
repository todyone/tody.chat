use yew::prelude::*;

enum Scene {
    Dashboard,
    Channel(String),
    AddChannel,
}

pub struct Chat {
    link: ComponentLink<Self>,
    scene: Scene,
}

pub enum Msg {
    SwitchTo(Scene),
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            scene: Scene::Dashboard,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SwitchTo(scene) => {
                self.scene = scene;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="app">
                <div class="user">
                    <p>{ "User" }</p>
                </div>
                <div class="channels">
                    <p>{ "Channels" }</p>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::AddChannel))>{ "Add Channel" }</button>
                </div>
                <div class="header">
                    <p>{ "Header" }</p>
                </div>
                <div class="content">
                    { self.view_content() }
                </div>
                <div class="message">
                    <p>{ "Message" }</p>
                </div>
            </div>
        }
    }
}

impl Chat {
    fn view_content(&self) -> Html {
        match self.scene {
            Scene::Dashboard => {
                html! {
                    <p>{ "Dashboard" }</p>
                }
            }
            Scene::Channel(_) => {
                html! {
                    <p>{ "Channel" }</p>
                }
            }
            Scene::AddChannel => {
                html! {
                    <p>{ "Add Channel" }</p>
                }
            }
        }
    }
}
