use crate::store;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Store(#[from] store::Error),
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
    #[error("Failed to deserialize: {0}")]
    JsonDeserialize(serde_json::Error),
    #[error("Failed to serialize: {0}")]
    JsonSerialize(#[from] serde_json::Error),
    #[error("Failed to deserialize timestamp: {0}")]
    TimeDeserialize(#[from] std::num::ParseIntError),
    #[error("Failed to serialize timestamp: {0}")]
    TimeSerialize(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] gotham::hyper::http::Error),
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
            Self::JsonDeserialize(_)
            | Self::TimeDeserialize(_)
            | Self::BadHeader
            | Self::Store(StoreError::Constraint) => StatusCode::BAD_REQUEST,
            Self::OutOfSync => StatusCode::PRECONDITION_FAILED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::ContentLengthMissing => StatusCode::LENGTH_REQUIRED,
            Self::ReadTimeout => StatusCode::REQUEST_TIMEOUT,
            Self::JsonSerialize(_)
            | Self::TimeSerialize(_)
            | Self::Http(_)
            | Self::Hyper(_)
            | Self::Store(
                StoreError::Io(_)
                | StoreError::Serde(_)
                | StoreError::Lock
                | StoreError::BadMillis(_)
                | StoreError::Sql(_),
            ) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_handler_error(self) -> gotham::handler::HandlerError {
        let status = self.status_code();
        gotham::handler::HandlerError::from(self).with_status(status)
    }
}
