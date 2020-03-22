use serde::{Deserialize, Serialize};

pub type Username = String;
// TODO: Use a wrapper that hides value for `Debug` and `Display`
pub type Password = String;

pub type ChannelName = String;

pub type Key = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    CreateSession(Credentials),
    RestoreSession(Key),
    CreateChannel(ChannelName),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {
    LoginUpdate(LoginUpdate),
    ChannelUpdate(ChannelUpdate),
    // TODO: Consider to listen to Channels List Notifications only (no specific mgs)
    ChannelCreated(ChannelName),
    Fail {
        reason: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum LoginUpdate {
    LoggedIn {
        key: Key,
    },
    LoginFail,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelUpdate {
    ChannelsList {
        channels: Vec<ChannelInfo>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChannelInfo {
    pub title: String,
}
