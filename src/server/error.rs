use crate::store;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Store(store::Error),
    #[error("Bad header")]
    BadHeader,
    #[error("Missing user header")]
    MissingUser,
    #[error("Hyper error: {0}")]
    Hyper(gotham::hyper::Error),
    #[error("Content length missing")]
    ContentLengthMissing,
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Read timeout")]
    ReadTimeout,
    #[error("Failed to acquire lock")]
    FailedToAcquireLock,
    #[error("Failed to deserialize: {0}")]
    JsonDeserialize(serde_json::Error),
    #[error("Failed to serialize: {0}")]
    JsonSerialize(serde_json::Error),
    #[error("Failed to deserialize timestamp: {0}")]
    TimeDeserialize(std::num::ParseIntError),
    #[error("Failed to serialize timestamp: {0}")]
    TimeSerialize(std::io::Error),
    #[error("HTTP error: {0}")]
    Http(gotham::hyper::http::Error),
    #[error("Client request is out of sync")]
    OutOfSync,
}

impl Error {
    fn status_code(&self) -> gotham::hyper::StatusCode {
        use gotham::hyper::StatusCode;
        use store::Error as StoreError;

        match self {
            Self::Store(StoreError::NotFound(_)) => StatusCode::NOT_FOUND,
            Self::Store(StoreError::StoreFull) => StatusCode::INSUFFICIENT_STORAGE,
            Self::MissingUser | Self::Store(StoreError::NoSuchUser(_)) => StatusCode::FORBIDDEN,
            Self::JsonDeserialize(_) | Self::TimeDeserialize(_) | Self::BadHeader => {
                StatusCode::BAD_REQUEST
            }
            Self::OutOfSync => StatusCode::PRECONDITION_FAILED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::ContentLengthMissing => StatusCode::LENGTH_REQUIRED,
            Self::ReadTimeout => StatusCode::REQUEST_TIMEOUT,
            Self::FailedToAcquireLock
            | Self::JsonSerialize(_)
            | Self::TimeSerialize(_)
            | Self::Http(_)
            | Self::Hyper(_)
            | Self::Store(StoreError::Io(_) | StoreError::Serde(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonSerialize(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::TimeSerialize(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::TimeDeserialize(e)
    }
}
