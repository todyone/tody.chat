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
                <div class="user">
                    <p>{ "User" }</p>
                </div>
                <div class="channels">
                    <p>{ "Channels" }</p>
                </div>
                <div class="header">
                    <p>{ "Header" }</p>
                </div>
                <div class="chat">
                    <p>{ "Chat" }</p>
                </div>
                <div class="message">
                    <p>{ "Message" }</p>
                </div>
            </div>
        }
    }
}
