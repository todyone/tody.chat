use serde::{Deserialize, Serialize};

pub type Username = String;
// TODO: Use a wrapper that hides value for `Debug` and `Display`
pub type Password = String;

pub type Key = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    Login(Credentials),
    RestoreSession(Key),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {
    LoggedIn {
        key: Key,
    },
    LoginFail,
    Fail(String),
}
