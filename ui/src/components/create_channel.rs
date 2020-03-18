use yew::prelude::*;

pub struct CreateChannel {}

pub enum Msg {}

impl Component for CreateChannel {
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
            <div>
                <label>{ "Channel name" }</label>
                <input placeholder="Channel name" />
            </div>
        }
    }
}
