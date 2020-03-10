use anyhow::Error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use tokio::task::spawn_blocking as wait;

#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        let pool = wait(|| {
            let manager = SqliteConnectionManager::memory();
            Pool::new(manager)
        }).await??;
        let mut this = Self { pool };
        this.initialize().await?;
        Ok(this)
    }

    async fn initialize(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn execute(&mut self, query: &str) -> Result<(), Error> {
        log::trace!("Executing query:\n{}", query);
        let pool = self.pool.get()?;
        wait(|| {
            pool.execute(query, params![]);
        }).await?;
        Ok(())
    }
}
