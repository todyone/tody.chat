pub mod delta;
pub mod types;

use crate::types::*;
use serde::{Deserialize, Serialize};

/// `Action`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {
    CreateSession(Credentials),
    RestoreSession(Key),
    CreateChannel(ChannelName),
}

/// `Notification`
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {
    Session(delta::SessionDelta),
}
