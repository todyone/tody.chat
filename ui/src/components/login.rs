use yew::prelude::*;

pub struct Login {
}

pub enum Msg {
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div><p>{ "Login" }</p></div>
        }
    }
}
