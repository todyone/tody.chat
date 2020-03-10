use crate::types::Username;
use anyhow::Error;
use rusqlite::{params, Connection, ToSql};

pub struct Dba {
    conn: Connection,
}

impl Dba {
    pub fn open() -> Result<Self, Error> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn initialize(&mut self) -> Result<(), Error> {
        self.create_tables()?;
        Ok(())
    }

    fn execute(&mut self, query: &str, params: &[&dyn ToSql]) -> Result<(), Error> {
        self.conn.execute(query, params)?;
        Ok(())
    }

    fn create_tables(&mut self) -> Result<(), Error> {
        log::debug!("Creating tables...");
        self.create_users_table()?;
        self.create_channels_table()?;
        Ok(())
    }

    fn create_users_table(&mut self) -> Result<(), Error> {
        log::debug!("Creating users table");
        self.execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                password TEXT,
                email TEXT
            )",
            params![],
        )
    }

    fn create_channels_table(&mut self) -> Result<(), Error> {
        log::debug!("Creating channels table");
        self.execute(
            "CREATE TABLE channels (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            params![],
        )
    }

    fn create_members_table(&mut self) -> Result<(), Error> {
        log::debug!("Creating members table");
        self.execute(
            "CREATE TABLE members (
                id INTEGER PRIMARY KEY,
                FOREIGN KEY (channel_id)
                    REFERENCES channels (id),
                FOREIGN KEY (user_id)
                    REFERENCES users (id)
            )",
            params![],
        )
    }

    pub fn create_user(&mut self, username: Username) -> Result<(), Error> {
        log::trace!("Creating user: {}", username);
        self.execute(
            "INSERT INTO users (username) VALUES (?)",
            params![&username],
        )?;
        Ok(())
    }
}
