use crate::network::{CodecError, ProtocolCodec};
use serde::{Deserialize, Serialize};

pub type ClientProtocol = ProtocolCodec<ClientToController, ControllerToClient>;

pub type ControllerProtocol = ProtocolCodec<ControllerToClient, ClientToController>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientToController {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerToClient {}
