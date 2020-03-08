use crate::network::ProtocolCodec;
use crate::types::{Password, Username};
use serde::{Deserialize, Serialize};

pub type ClientProtocol = ProtocolCodec<ClientToController, ControllerToClient>;

pub type ControllerProtocol = ProtocolCodec<ControllerToClient, ClientToController>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientToController {
    CreateUser {
        username: Username,
    },
    SetPassword {
        username: Username,
        password: Password,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerToClient {}
