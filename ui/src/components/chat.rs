use yew::prelude::*;

pub struct Chat {}

pub enum Msg {}

impl Component for Chat {
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
