use yew::prelude::*;

pub struct App {}

pub enum Msg {}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="app">
                <div class="navbar">
                    <div class="team">
                        <p>{ "Tody.Chat" }</p>
                    </div>
                    <div class="channel">
                    </div>
                    <div class="search">
                    </div>
                </div>
                <div class="sidebar">
                </div>
                <div class="mainview">
                </div>
            </div>
        }
    }
}
