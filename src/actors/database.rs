// TODO: Rewrite this module to fully async
// when SQLite crates will support that.

use crate::db::Dba;
use crate::types::{Password, Username};
use anyhow::Error;
use async_trait::async_trait;
use futures::{select, StreamExt};
use meio::{wrapper, Actor, Context};
use tokio::task::block_in_place as wait;

wrapper!(Database for DatabaseActor);

impl Database {
    pub fn start() -> Self {
        let actor = DatabaseActor { dba: None };
        meio::spawn(actor)
    }

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

pub struct DatabaseActor {
    dba: Option<Dba>,
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
impl Actor for DatabaseActor {
    type Message = Msg;
    type Interface = Database;

    fn generic_name() -> &'static str {
        "Database"
    }

    async fn routine(&mut self, ctx: Context<Self>) -> Result<(), Error> {
        self.run(ctx).await
    }
}

/// Messagning routines.
impl DatabaseActor {
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
        let dba = Dba::open()?;
        self.dba = Some(dba);
        wait(|| self.dba().initialize())?;
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
            Msg::CreateUser { username } => {
                wait(|| self.dba().create_user(username))?;
            }
            Msg::SetPassword { .. } => {}
        }
        Ok(())
    }
}

/// DatabaseActor routines.
impl DatabaseActor {
    fn dba(&mut self) -> &mut Dba {
        self.dba.as_mut().expect("DBA lost")
    }
}
