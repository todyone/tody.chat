use serde::{Deserialize, Serialize};

/// Actions
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    Session(actions::Session),
    Channel(actions::Channel),
}

pub mod actions {
    use crate::types::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Session {
        Create(Credentials),
        Restore(Key),
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Channel {
        Create(ChannelName),
    }
}
