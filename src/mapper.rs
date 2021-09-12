#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to deserialize: {0}")]
    Deserialize(serde_json::Error),
    #[error("Hyper error: {0}")]
    Hyper(gotham::hyper::Error),
}

impl From<gotham::hyper::Error> for Error {
    fn from(e: gotham::hyper::Error) -> Self {
        Self::Hyper(e)
    }
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
        use gotham::hyper::{body, Body};
        use gotham::state::FromState;

        let body = body::to_bytes(Body::take_from(state)).await?;
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
