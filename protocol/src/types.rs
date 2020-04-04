use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! basic_type {
    ( $name:ident ) => {
        #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

basic_type!(Username);

basic_type!(Password);

basic_type!(Key);

basic_type!(ChannelName);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Credentials {
    pub username: Username,
    pub password: Password,
}
