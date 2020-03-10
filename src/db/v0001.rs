use crate::types::{Id, Password, Username};
use rusqlite::{params, Connection};
use thiserror::Error;

pub struct User {
    pub id: Id,
    pub username: Username,
    pub password: Password,
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
        self.create_channels_table()?;
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
            "INSERT INTO users (username) VALUES (?)",
            params![&username],
        )?;
        Ok(())
    }

    pub fn get_user(&mut self, username: Username) -> Result<(), DbaError> {
        log::trace!("Getting user: {}", username);
        self.conn.execute(
            "SELECT id, username, password, email FROM users WHERE username = ?",
            params![&username],
        )?;
        Ok(())
    }
}
