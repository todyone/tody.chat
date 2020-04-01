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

/// `Action`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    CreateSession(Credentials),
    RestoreSession(Key),
    CreateChannel(ChannelName),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {
    Delta(Delta),
    Reaction(Reaction),
}

/// `Notification`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Delta {
    LoginUpdate(LoginUpdate),
    ChannelUpdate(ChannelUpdate),
}

/// `Reaction`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Reaction {
    Success,
    Fail {
        reason: String,
    },
}

impl Reaction {
    pub fn fail(reason: impl ToString) -> Self {
        Self::Fail {
            reason: reason.to_string(),
        }
    }
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
