use yew::prelude::*;

enum Scene {
    Splash,
}

pub struct App {
    scene: Scene,
    link: ComponentLink<Self>,
}

pub enum Msg {
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            scene: Scene::Splash,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        match self.scene {
            Scene::Splash => {
                html! {
                    <div />
                }
            }
        }
    }
}

