use super::Id;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User not found `{0}`")]
    NoSuchUser(String),
    #[error("Entry not found for id `{0}`")]
    NotFound(Id),
    #[error("Store full")]
    StoreFull,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Serde error: {0}")]
    Serde(String),
    #[error("Failed to acquire lock")]
    Lock,
    #[error("{0}")]
    Sql(sqlx::Error),
    #[error("Failed to parse timestamp: {0}")]
    BadMillis(#[from] std::num::TryFromIntError),
    #[error("Failed constraint")]
    Constraint,
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err)
                if db_err
                    .try_downcast_ref::<sqlx::sqlite::SqliteError>()
                    .is_some()
                    && db_err.message().starts_with("FOREIGN KEY") =>
            {
                Self::Constraint
            }
            _ => Self::Sql(err),
        }
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        Self::Lock
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        Self::Lock
    }
}
