use serde::{Deserialize, Serialize};

/// Actions
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    Info(actions::Info),
    User(actions::User),
    Session(actions::Session),
    Channel(actions::Channel),
}

pub mod actions {
    use crate::types::*;
    use super::ClientToServer;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Info {
        Subscribe(InfoType),
        Unsubscribe(InfoType),
    }

    impl Into<ClientToServer> for Info {
        fn into(self) -> ClientToServer {
            ClientToServer::Info(self)
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum InfoType {
        User(Option<Username>),
        Session(Option<Username>),
        Channel(Option<ChannelName>),
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum User {
        Create(Username),
        Delete(Username),
    }

    impl Into<ClientToServer> for User {
        fn into(self) -> ClientToServer {
            ClientToServer::User(self)
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Session {
        Create(Credentials),
        Restore(Key),
    }

    impl Into<ClientToServer> for Session {
        fn into(self) -> ClientToServer {
            ClientToServer::Session(self)
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Channel {
        Create(ChannelName),
        Delete(ChannelName),
    }

    impl Into<ClientToServer> for Channel {
        fn into(self) -> ClientToServer {
            ClientToServer::Channel(self)
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Message {
        Create,
        Delete,
    }
}
