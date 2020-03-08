use crate::agents::{Action, Connector, Notification};
use protocol::Credentials;
use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    connector: Box<dyn Bridge<Connector>>,
    username: String,
    password: String,
}

pub enum Msg {
    UpdateUsername(String),
    UpdatePassword(String),
    SendCredentials,
    Notification(Notification),
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|n| Msg::Notification(n));
        let connector = Connector::bridge(callback);
        Self {
            link,
            connector,
            username: String::new(),
            password: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateUsername(username) => {
                self.username = username;
            }
            Msg::UpdatePassword(password) => {
                self.password = password;
            }
            Msg::SendCredentials => {
                let creds = Credentials {
                    username: self.username.clone(),
                    password: self.password.clone(),
                };
                let action = Action::SetCredentials(creds);
                self.connector.send(action);
            }
            Msg::Notification(_) => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>
                    <label>{ "Username" }</label>
                    <input value=self.username
                           placeholder="Username"
                           oninput=self.link.callback(|e: InputData| Msg::UpdateUsername(e.value)) />
                </div>
                <div>
                    <label>{ "Password" }</label>
                    <input type="password"
                           value=self.password
                           placeholder="Password"
                           oninput=self.link.callback(|e: InputData| Msg::UpdatePassword(e.value)) />
                </div>
                <p onclick=self.link.callback(|_| Msg::SendCredentials)>{ "Login" }</p>
            </div>
        }
    }
}
