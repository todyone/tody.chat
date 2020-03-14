use crate::types::{Id, Password, Username};
use rusqlite::{params, Connection, Row};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Id,
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Id,
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
            "CREATE TABLE users (
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
            "CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                key TEXT NOT UNLL UNIQUE,
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
            "CREATE TABLE channels (
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
            "CREATE TABLE members (
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

    pub fn create_session(&mut self, user_id: Id) -> Result<(), DbaError> {
        log::trace!("Creating session for: {}", user_id);
        todo!();
    }

    pub fn get_session(&mut self, user_id: Id) -> Result<Session, DbaError> {
        log::trace!("Getting session: {}", user_id);
        todo!();
    }

    pub fn set_password(&mut self, username: Username, password: Password) -> Result<(), DbaError> {
        log::trace!("Setting password for user: {}", username);
        self.conn.execute(
            "UPDATE users SET password = ? WHERE username = ?",
            params![&password, &username],
        )?;
        Ok(())
    }

    pub fn find_user(&mut self, username: Username) -> Result<Option<User>, DbaError> {
        log::trace!("Getting user: {}", username);
        let user = self.conn.query_row(
            "SELECT id, username, password, email FROM users WHERE username = ?",
            params![&username],
            |row| User::try_from(row),
        );
        log::trace!("Find user result: {:?}", user);
        match user {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(DbaError::DbError(err)),
            Ok(user) => Ok(Some(user)),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
