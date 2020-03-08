use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ClientToServer {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ServerToClient {}
