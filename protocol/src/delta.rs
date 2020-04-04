use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SessionDelta {
    LoggedIn { key: Key },
    LoginFailed,
}
