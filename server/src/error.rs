use store::Error as StoreError;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("Failed to serialize payload: {0}")]
    Serialize(String),
    #[error("Failed to deserialize payload: {0}")]
    Deserialize(String),
}
