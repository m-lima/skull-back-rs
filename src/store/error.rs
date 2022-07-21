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
    FailedToAcquireLock,
    #[error("{0}")]
    Sql(#[from] sqlx::Error),
    #[error("Failed to parse timestamp:{0}")]
    BadMillis(#[from] std::num::TryFromIntError),
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        Self::FailedToAcquireLock
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        Self::FailedToAcquireLock
    }
}
