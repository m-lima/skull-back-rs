use super::mapper;
use crate::store;

// TODO: Should this live here or on middleware?
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Store(store::Error),
    #[error("{0}")]
    Mapper(mapper::Error),
    #[error("Failed to acquire lock")]
    FailedToAcquireLock,
    #[error("Failed to serialize: {0}")]
    Serialize(serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(gotham::hyper::http::Error),
}

impl Error {
    fn status_code(&self) -> gotham::hyper::StatusCode {
        use gotham::hyper::StatusCode;
        match self {
            Self::Store(store::Error::NotFound(_)) => StatusCode::NOT_FOUND,
            Self::Store(store::Error::StoreFull) => StatusCode::INSUFFICIENT_STORAGE,
            Self::Mapper(mapper::Error::Deserialize(_)) => StatusCode::BAD_REQUEST,
            Self::Mapper(mapper::Error::PayloadTooLarge) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Mapper(mapper::Error::ContentLengthMissing) => StatusCode::LENGTH_REQUIRED,
            Self::Mapper(mapper::Error::ReadTimeout) => StatusCode::REQUEST_TIMEOUT,
            Self::FailedToAcquireLock
            | Self::Serialize(_)
            | Self::Http(_)
            | Self::Mapper(mapper::Error::Hyper(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_handler_error(self) -> gotham::handler::HandlerError {
        let status = self.status_code();
        gotham::handler::HandlerError::from(self).with_status(status)
    }
}

impl From<store::Error> for Error {
    fn from(e: store::Error) -> Self {
        Self::Store(e)
    }
}

impl From<mapper::Error> for Error {
    fn from(e: mapper::Error) -> Self {
        Self::Mapper(e)
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, dyn store::Store>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, dyn store::Store>>) -> Self {
        Self::FailedToAcquireLock
    }
}

impl From<gotham::hyper::http::Error> for Error {
    fn from(e: gotham::hyper::http::Error) -> Self {
        Self::Http(e)
    }
}
