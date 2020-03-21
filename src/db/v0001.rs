use crate::types::{Channel, Id, Password, Username};
use protocol::Key;
use rusqlite::{params, Connection, Row};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Id,
    pub username: Username,
    pub password: Password,
}

impl User {
    const SELECT_BY_NAME: &'static str =
        "SELECT id, username, password, email FROM users WHERE username = ?";
}

impl TryFrom<&Row<'_>> for User {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Id,
    pub key: Key,
    pub user_id: Id,
}

impl Session {
    const SELECT_BY_KEY: &'static str = "SELECT id, key, user_id FROM sessions WHERE key = ?";
}

impl TryFrom<&Row<'_>> for Session {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            key: row.get(1)?,
            user_id: row.get(2)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChannelRecord {
    pub id: Id,
    pub channel: Channel,
}

impl ChannelRecord {
    const SELECT_BY_NAME: &'static str = "SELECT id, name FROM channels WHERE name = ?";
}

impl TryFrom<&Row<'_>> for ChannelRecord {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            channel: row.get(1)?,
        })
    }
}

#[derive(Error, Debug)]
pub enum DbaError {
    #[error("db error: {0}")]
    DbError(#[from] rusqlite::Error),
}

pub struct Dba {
    conn: Connection,
}

impl Dba {
    #[cfg(not(test))]
    pub fn open() -> Result<Self, DbaError> {
        let path = "data/v0001.db3";
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub fn open() -> Result<Self, DbaError> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn initialize(&mut self) -> Result<(), DbaError> {
        self.create_tables()?;
        Ok(())
    }

    fn create_tables(&mut self) -> Result<(), DbaError> {
        log::debug!("Creating tables...");
        self.create_users_table()?;
        self.create_sessions_table()?;
        self.create_channels_table()?;
        self.create_members_table()?;
        Ok(())
    }

    fn create_users_table(&mut self) -> Result<(), DbaError> {
        log::debug!("Creating users table");
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                password TEXT,
                email TEXT
            )",
            params![],
        )?;
        Ok(())
    }

    fn create_sessions_table(&mut self) -> Result<(), DbaError> {
        log::debug!("Creating sessions table");
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES users (id)
            )",
            params![],
        )?;
        Ok(())
    }

    fn create_channels_table(&mut self) -> Result<(), DbaError> {
        log::debug!("Creating channels table");
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS channels (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            params![],
        )?;
        Ok(())
    }

    fn create_members_table(&mut self) -> Result<(), DbaError> {
        log::debug!("Creating members table");
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS members (
                id INTEGER PRIMARY KEY,
                channel_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (channel_id)
                    REFERENCES channels (id),
                FOREIGN KEY (user_id)
                    REFERENCES users (id)
            )",
            params![],
        )?;
        Ok(())
    }

    pub fn create_user(&mut self, username: Username) -> Result<(), DbaError> {
        log::trace!("Creating user: {}", username);
        self.conn.execute(
            "INSERT INTO users (username, password) VALUES (?, '')",
            params![&username],
        )?;
        Ok(())
    }

    pub fn set_password(&mut self, username: Username, password: Password) -> Result<(), DbaError> {
        log::trace!("Setting password for user: {}", username);
        self.conn.execute(
            "UPDATE users SET password = ? WHERE username = ?",
            params![&password, &username],
        )?;
        Ok(())
    }

    pub fn get_user(&mut self, name: Username) -> Result<User, DbaError> {
        log::trace!("Getting user: {}", name);
        let value = self
            .conn
            .query_row(User::SELECT_BY_NAME, params![&name], |row| {
                User::try_from(row)
            });
        log::trace!("Find user result: {:?}", value);
        value.map_err(DbaError::from)
    }

    pub fn create_session(&mut self, user_id: Id, key: Key) -> Result<(), DbaError> {
        log::trace!("Creating session for: {}", user_id);
        self.conn.execute(
            "INSERT INTO sessions (user_id, key) VALUES (?, ?)",
            params![&user_id, &key],
        )?;
        Ok(())
    }

    pub fn get_session(&mut self, key: Key) -> Result<Session, DbaError> {
        log::trace!("Getting sessions: {}", key);
        let value = self
            .conn
            .query_row(Session::SELECT_BY_KEY, params![&key], |row| {
                Session::try_from(row)
            });
        log::trace!("Find sessions result: {:?}", value);
        value.map_err(DbaError::from)
    }

    pub fn create_channel(&mut self, name: Channel) -> Result<(), DbaError> {
        log::trace!("Creating channel named: {}", name);
        self.conn
            .execute("INSERT INTO channels (name) VALUES (?)", params![&name])?;
        Ok(())
    }

    pub fn get_channel(&mut self, name: Channel) -> Result<ChannelRecord, DbaError> {
        log::trace!("Getting channel: {}", name);
        let value = self
            .conn
            .query_row(ChannelRecord::SELECT_BY_NAME, params![&name], |row| {
                ChannelRecord::try_from(row)
            });
        log::trace!("Find channel result: {:?}", value);
        value.map_err(DbaError::from)
    }

    pub fn add_member(&mut self, channel_id: Id, user_id: Id) -> Result<(), DbaError> {
        log::trace!("Add user {} to channel {}", user_id, channel_id);
        self.conn.execute(
            "INSERT INTO members (channel_id, user_id) VALUES (?, ?)",
            params![&channel_id, user_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::ops::{Deref, DerefMut};

    struct TestDba {
        dba: Dba,
    }

    impl Deref for TestDba {
        type Target = Dba;

        fn deref(&self) -> &Self::Target {
            &self.dba
        }
    }

    impl DerefMut for TestDba {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.dba
        }
    }

    impl TestDba {
        fn new() -> Result<Self, DbaError> {
            let mut dba = Dba::open()?;
            dba.initialize()?;
            let this = Self { dba };
            Ok(this)
        }

        fn create_test_user(&mut self) -> Result<Id, DbaError> {
            // TODO: Generate random name later
            let username = Username::from("username");
            self.dba.create_user(username.clone())?;
            let record = self.dba.get_user(username.clone())?;
            Ok(record.id)
        }

        fn create_test_channel(&mut self) -> Result<Id, DbaError> {
            let channel = Channel::from("channel-1");
            self.dba.create_channel(channel.clone())?;
            let record = self.dba.get_channel(channel.clone())?;
            Ok(record.id)
        }
    }

    fn dba() -> Result<Dba, DbaError> {
        let mut dba = Dba::open()?;
        dba.initialize()?;
        Ok(dba)
    }

    #[test]
    fn user_creation() -> Result<(), DbaError> {
        let username = Username::from("username");
        let password = Password::from("password");
        let mut dba = dba()?;
        dba.create_user(username.clone())?;
        dba.set_password(username.clone(), password.clone())?;
        let user = dba.get_user(username.clone())?.expect("user not found");
        assert_eq!(user.username, username);
        assert_eq!(user.password, password);
        Ok(())
    }

    #[test]
    fn user_not_exists() -> Result<(), DbaError> {
        let username = Username::from("username");
        let mut dba = dba()?;
        let user = dba.get_user(username.clone())?;
        assert!(user.is_none());
        Ok(())
    }

    #[test]
    fn session_check() -> Result<(), DbaError> {
        let username = Username::from("username");
        let key = Key::from("key");
        let mut dba = dba()?;
        dba.create_user(username.clone())?;
        let user = dba
            .get_user(username.clone())?
            .expect("user hadn't created");
        dba.create_session(user.id, key.clone())?;
        let session = dba.get_session(key.clone())?.expect("session not found");
        assert_eq!(session.key, key);
        Ok(())
    }

    #[test]
    fn channel_check() -> Result<(), DbaError> {
        let channel = Channel::from("channel-1");
        let mut dba = TestDba::new()?;
        dba.create_channel(channel)?;
        Ok(())
    }

    #[test]
    fn channel_membership() -> Result<(), DbaError> {
        let mut dba = TestDba::new()?;
        let user_id = dba.create_test_user()?;
        let channel_id = dba.create_test_channel()?;
        // TODO: Replace Id with separated ChannelId and UserId.
        // It's possible to confuse them today.
        dba.add_member(channel_id, user_id)?;
        // TODO: Check member.
        Ok(())
    }
}
