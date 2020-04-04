use serde::{Deserialize, Serialize};

/// Deltas - notifications
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {
    Session(deltas::Session),
}

pub mod deltas {
    use crate::types::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub enum Session {
        LoggedIn { key: Key },
        LoginFailed,
    }
}
