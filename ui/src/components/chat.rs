use crate::components::CreateChannel;
use yew::prelude::*;

#[derive(Debug, Clone)]
enum Scene {
    Dashboard,
    Channel(String),
    AddChannel,
}

pub struct Chat {
    link: ComponentLink<Self>,
    scene: Scene,
    // TODO: Consider to use a size-limited stack here
    previous_scene: Option<Scene>,
}

#[derive(Debug)]
pub enum Msg {
    SwitchTo(Scene),
    Back,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            scene: Scene::Dashboard,
            previous_scene: Some(Scene::Dashboard),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log::debug!("Chat msg: {:?}", msg);
        match msg {
            Msg::SwitchTo(scene) => {
                self.scene = scene;
                match self.scene {
                    Scene::Dashboard | Scene::Channel(_) => {
                        self.previous_scene = Some(self.scene.clone());
                    }
                    _ => {}
                }
            }
            Msg::Back => {
                log::debug!("Switching back: {:?}", self.previous_scene);
                if let Some(scene) = self.previous_scene.take() {
                    self.scene = scene;
                }
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
                    <CreateChannel oncomplete=self.link.callback(|_| Msg::Back) />
                }
            }
        }
    }
}
