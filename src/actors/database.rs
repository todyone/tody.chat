// TODO: Rewrite this module to fully async
// when SQLite crates will support that.

use crate::types::{Password, Username};
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, StreamExt};
use meio::{wrapper, Actor, Address, Context, Wrapper};
use rusqlite::{params, Connection};
use tokio::task::block_in_place as wait;

wrapper!(DatabaseWrapper for Database);

impl DatabaseWrapper {
    pub async fn create_user(&mut self, username: Username) -> Result<(), Error> {
        self.send(Msg::CreateUser { username }).await
    }

    pub async fn set_password(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<(), Error> {
        self.send(Msg::SetPassword { username, password }).await
    }
}

pub struct Database {
    conn: Option<Connection>,
}

impl Database {
    pub fn new() -> Self {
        Self { conn: None }
    }
}

pub enum Msg {
    CreateUser {
        username: Username,
    },
    SetPassword {
        username: Username,
        password: Password,
    },
}

#[async_trait]
impl Actor for Database {
    type Message = Msg;
    type Interface = DatabaseWrapper;

    fn generic_name() -> &'static str {
        "Database"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await
    }
}

/// Messagning routines.
impl Database {
    /// `select!` macro can't be used in `routine` directly because of:
    /// ```
    /// error[E0434]: can't capture dynamic environment in a fn item
    ///  --> src/actors/database.rs:40:25
    ///   |
    ///   |                         self.process_message(msg).await?;
    ///   |                         ^^^^
    ///   |
    ///   = help: use the `|| { ... }` closure form instead
    ///   = note: this error originates in a macro (in Nightly builds, run with -Z macro-backtrace for more info)
    /// ```
    async fn run(&mut self, mut ctx: Context<Self>) -> Result<(), Error> {
        self.open_database().await?;
        self.create_tables().await?;
        loop {
            select! {
                msg = ctx.rx.next() => {
                    if let Some(msg) = msg {
                        self.process_message(msg).await?;
                    } else {
                        log::trace!("Consumer of database closed. Terminating database actor...");
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_message(&mut self, msg: Msg) -> Result<(), Error> {
        match msg {
            Msg::CreateUser { .. } => {}
            Msg::SetPassword { .. } => {}
        }
        Ok(())
    }
}

/// Database routines.
impl Database {
    /// Just unwraps a reference to a `Connection`.
    fn db(&mut self) -> &mut Connection {
        self.conn.as_mut().expect("Database connection lost")
    }

    async fn open_database(&mut self) -> Result<(), Error> {
        log::debug!("Connecting to a database...");
        let conn = wait(Connection::open_in_memory)?;
        self.conn = Some(conn);
        Ok(())
    }

    async fn execute(&mut self, query: &str) -> Result<(), Error> {
        log::trace!("Executing query:\n{}", query);
        wait(|| self.db().execute(query, params![]))
            .map(drop)
            .map_err(Error::from)
    }

    async fn create_tables(&mut self) -> Result<(), Error> {
        log::debug!("Creating tables...");
        self.execute(CREATE_PERSON_TABLE).await?;
        Ok(())
    }
}

const CREATE_PERSON_TABLE: &str = "CREATE TABLE users (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    time_created    TEXT NOT NULL,
    data            BLOB
)";
