// TODO: Use wrapper over primitives

use rusqlite::{
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
    Result, ToSql,
};

/// Internal alias
type Id = u32;

macro_rules! id {
    ( $name:ident ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(Id);

        impl ToSql for $name {
            fn to_sql(&self) -> Result<ToSqlOutput> {
                self.0.to_sql()
            }
        }

        impl FromSql for $name {
            fn column_result(value: ValueRef) -> FromSqlResult<Self> {
                Id::column_result(value).map(Self)
            }
        }
    };
}

id!(UserId);
id!(SessionId);
id!(ChannelId);

pub type Username = String;
pub type Password = String;
pub type ChannelName = String;
