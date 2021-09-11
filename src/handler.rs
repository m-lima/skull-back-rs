use crate::store;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    Store(store::Error),
    #[error("Failed to acquire lock")]
    FailedToAcquireLock,
    #[error("Failed to serialize")]
    Serde,
}

impl Error {
    fn status_code(&self) -> gotham::hyper::StatusCode {
        match self {
            Self::Store(store::Error::NotFound(_)) => gotham::hyper::StatusCode::NOT_FOUND,
            Self::Store(store::Error::StoreFull) => gotham::hyper::StatusCode::INSUFFICIENT_STORAGE,
            Self::FailedToAcquireLock | Self::Serde => {
                gotham::hyper::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn into_response(
        self,
        state: &gotham::state::State,
    ) -> gotham::hyper::Response<gotham::hyper::Body> {
        let status = self.status_code();
        log::warn!("{}", &self);
        gotham::helpers::http::response::create_empty_response(state, status)
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

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::Serde
    }
}

#[derive(serde::Serialize)]
struct Payload<'a, P: serde::Serialize> {
    id: store::Id,
    #[serde(flatten)]
    payload: &'a P,
}

impl<'a, P: serde::Serialize> Payload<'a, P> {
    fn build(pair: &(&'a store::Id, &'a P)) -> Self {
        Self {
            id: *pair.0,
            payload: pair.1,
        }
    }
}

pub mod skull {
    use super::Error;
    use super::Payload;
    use crate::middleware;
    use crate::router;
    use crate::store;

    pub fn get(
        mut state: gotham::state::State,
    ) -> (
        gotham::state::State,
        gotham::hyper::Response<gotham::hyper::Body>,
    ) {
        fn inner(id: store::Id, store: &mut middleware::Store) -> Result<String, Error> {
            let mut store = store.get()?;
            let skull = store.skull().read(id)?;
            let json = serde_json::to_string(skull)?;
            Ok(json)
        }

        use gotham::handler::IntoResponse;
        use gotham::state::FromState;

        let id = router::IdExtractor::take_from(&mut state).id();
        let store = middleware::Store::borrow_mut_from(&mut state);

        let response =
            inner(id, store).map_or_else(|e| e.into_response(&state), |r| r.into_response(&state));

        (state, response)
    }

    pub fn get_all(
        mut state: gotham::state::State,
    ) -> (
        gotham::state::State,
        gotham::hyper::Response<gotham::hyper::Body>,
    ) {
        fn inner(store: &mut middleware::Store) -> Result<String, Error> {
            let mut store = store.get()?;
            let skull = store
                .skull()
                .list()?
                .iter()
                .map(Payload::build)
                .collect::<Vec<_>>();
            let json = serde_json::to_string(&skull)?;
            Ok(json)
        }

        use gotham::handler::IntoResponse;
        use gotham::state::FromState;

        let store = middleware::Store::borrow_mut_from(&mut state);

        let response =
            inner(store).map_or_else(|e| e.into_response(&state), |r| r.into_response(&state));

        (state, response)
    }
}
