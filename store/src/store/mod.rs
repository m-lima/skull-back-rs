pub mod occurrences;
pub mod quicks;
pub mod skulls;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Store {
    pool: sqlx::sqlite::SqlitePool,
}

impl Store {
    pub async fn new<P: AsRef<std::path::Path>>(path: P, max_connections: u32) -> Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(path.as_ref())
            .optimize_on_close(true, Some(1000))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn in_memory(max_connections: u32) -> Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .optimize_on_close(true, Some(1000))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result {
        sqlx::migrate!().run(&self.pool).await.map_err(Into::into)
    }

    #[must_use]
    pub fn skulls(&self) -> skulls::Skulls<'_> {
        skulls::Skulls::new(self)
    }

    #[must_use]
    pub fn quicks(&self) -> quicks::Quicks<'_> {
        quicks::Quicks::new(self)
    }

    #[must_use]
    pub fn occurrences(&self) -> occurrences::Occurrences<'_> {
        occurrences::Occurrences::new(self)
    }
}

fn check_non_empty<'a>(value: &'a str, field: &'static str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() || value.contains('\n') {
        Err(Error::InvalidParameter(field))
    } else {
        Ok(value)
    }
}
