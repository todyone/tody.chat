use yew::prelude::*;

pub struct Splash {}

pub enum Msg {}

impl Component for Splash {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="loader">
                <img src="/loader.svg" />
            </div>
        }
    }
}
