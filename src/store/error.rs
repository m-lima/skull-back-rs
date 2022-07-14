use super::Id;

#[allow(dead_code)]
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
}
