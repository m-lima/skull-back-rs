#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to deserialize: {0}")]
    Deserialize(serde_json::Error),
    #[error("Hyper error: {0}")]
    Hyper(gotham::hyper::Error),
    #[error("Content length missing")]
    ContentLengthMissing,
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Read timeout")]
    ReadTimeout,
}

pub mod request {
    use super::Error;
    use crate::store;

    #[derive(
        serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender,
    )]
    pub struct Id {
        pub id: store::Id,
    }

    pub async fn body<D: store::Data>(state: &mut gotham::state::State) -> Result<D, Error> {
        use gotham::hyper;
        use gotham::state::FromState;

        let request_length = hyper::HeaderMap::borrow_from(state)
            .get(hyper::header::CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok())
            .and_then(|len| len.parse::<usize>().ok())
            .ok_or(Error::ContentLengthMissing)?;

        if request_length > 1024 {
            return Err(Error::PayloadTooLarge);
        }

        // Hyper reads up to Content-Length. No need for chunk-wise verification
        let body = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            hyper::body::to_bytes(hyper::Body::borrow_mut_from(state)),
        )
        .await
        .map_err(|_| Error::ReadTimeout)?
        .map_err(Error::Hyper)?;
        serde_json::from_slice(&body).map_err(Error::Deserialize)
    }
}

pub mod respose {
    use crate::store;

    #[derive(serde::Serialize, Clone, Debug, PartialEq)]
    pub struct DataWithId<'a, D: store::Data> {
        id: store::Id,
        #[serde(flatten)]
        data: &'a D,
    }
}
