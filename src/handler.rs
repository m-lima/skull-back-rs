use crate::store;

// TODO: Should this live here or on middleware?
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Store(store::Error),
    #[error("Failed to acquire lock")]
    FailedToAcquireLock,
    #[error("Failed to deserialize: {0}")]
    Deserialize(serde_json::Error),
    #[error("Failed to serialize: {0}")]
    Serialize(serde_json::Error),
    #[error("Hyper error: {0}")]
    Hyper(gotham::hyper::Error),
    #[error("HTTP error: {0}")]
    Http(gotham::hyper::http::Error),
}

impl Error {
    fn status_code(&self) -> gotham::hyper::StatusCode {
        match self {
            Self::Store(store::Error::NotFound(_)) => gotham::hyper::StatusCode::NOT_FOUND,
            Self::Store(store::Error::StoreFull) => gotham::hyper::StatusCode::INSUFFICIENT_STORAGE,
            Self::Deserialize(_) => gotham::hyper::StatusCode::BAD_REQUEST,
            Self::FailedToAcquireLock | Self::Serialize(_) | Self::Http(_) | Self::Hyper(_) => {
                gotham::hyper::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn into_response(self) -> gotham::handler::HandlerError {
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

impl From<gotham::hyper::Error> for Error {
    fn from(e: gotham::hyper::Error) -> Self {
        Self::Hyper(e)
    }
}

impl From<gotham::hyper::http::Error> for Error {
    fn from(e: gotham::hyper::http::Error) -> Self {
        Self::Http(e)
    }
}

macro_rules! impl_handle {
    ($name:ty) => {
        impl $name {
            async fn wrap(mut state: gotham::state::State) -> gotham::handler::HandlerResult {
                match Self::handle(&mut state).await {
                    Ok(r) => Ok((state, r)),
                    Err(e) => Err((state, e.into_response())),
                }
            }
        }

        impl gotham::handler::Handler for $name {
            fn handle(
                self,
                state: gotham::state::State,
            ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
                Box::pin(Self::wrap(state))
            }
        }

        impl gotham::handler::NewHandler for $name {
            type Instance = Self;

            fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
                Ok(*self)
            }
        }
    };
}

pub mod skull {
    use super::Error;
    use crate::{middleware, router};

    #[derive(Copy, Clone)]
    pub struct List;

    impl List {
        async fn handle(
            state: &mut gotham::state::State,
        ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
            use gotham::state::FromState;

            let store = middleware::Store::borrow_mut_from(state);

            let json = {
                let mut store = store.get()?;
                let skulls = store.skull().list()?;
                serde_json::to_string(&skulls).map_err(Error::Serialize)?
            };

            let response = gotham::hyper::Response::builder()
                .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
                .status(gotham::hyper::StatusCode::OK)
                .body(gotham::hyper::Body::from(json))?;

            Ok(response)
        }
    }

    impl_handle!(List);

    #[derive(Copy, Clone)]
    pub struct Create;

    impl Create {
        async fn handle(
            state: &mut gotham::state::State,
        ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
            use gotham::hyper::{body, Body};
            use gotham::state::FromState;

            let skull = {
                let body = body::to_bytes(Body::take_from(state)).await?;
                serde_json::from_slice(&body).map_err(Error::Deserialize)?
            };
            let store = middleware::Store::borrow_mut_from(state);

            let id = {
                let mut store = store.get()?;
                store.skull().create(skull)?
            };

            let response = gotham::hyper::Response::builder()
                .header(gotham::hyper::header::LOCATION, id)
                .status(gotham::hyper::StatusCode::CREATED)
                .body(gotham::hyper::Body::empty())?;

            Ok(response)
        }
    }

    impl_handle!(Create);

    #[derive(Copy, Clone)]
    pub struct Read;

    impl Read {
        async fn handle(
            state: &mut gotham::state::State,
        ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
            use gotham::state::FromState;

            let id = router::IdExtractor::take_from(state).id();
            let store = middleware::Store::borrow_mut_from(state);

            let json = {
                let mut store = store.get()?;
                let skull = store.skull().read(id)?;
                serde_json::to_string(&skull).map_err(Error::Serialize)?
            };

            let response = gotham::hyper::Response::builder()
                .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
                .status(gotham::hyper::StatusCode::OK)
                .body(gotham::hyper::Body::from(json))?;

            Ok(response)
        }
    }

    impl_handle!(Read);

    #[derive(Copy, Clone)]
    pub struct Update;

    impl Update {
        async fn handle(
            state: &mut gotham::state::State,
        ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
            use gotham::hyper::{body, Body};
            use gotham::state::FromState;

            let skull = {
                let body = body::to_bytes(Body::take_from(state)).await?;
                serde_json::from_slice(&body).map_err(Error::Deserialize)?
            };
            let id = router::IdExtractor::take_from(state).id();
            let store = middleware::Store::borrow_mut_from(state);

            let json = {
                let mut store = store.get()?;
                let skull = store.skull().update(id, skull)?;
                serde_json::to_string(&skull).map_err(Error::Serialize)?
            };

            let response = gotham::hyper::Response::builder()
                .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
                .status(gotham::hyper::StatusCode::OK)
                .body(gotham::hyper::Body::from(json))?;

            Ok(response)
        }
    }

    impl_handle!(Update);

    #[derive(Copy, Clone)]
    pub struct Delete;

    impl Delete {
        async fn handle(
            state: &mut gotham::state::State,
        ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
            use gotham::state::FromState;

            let id = router::IdExtractor::take_from(state).id();
            let store = middleware::Store::borrow_mut_from(state);

            let json = {
                let mut store = store.get()?;
                let skull = store.skull().delete(id)?;
                serde_json::to_string(&skull).map_err(Error::Serialize)?
            };

            let response = gotham::hyper::Response::builder()
                .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
                .status(gotham::hyper::StatusCode::OK)
                .body(gotham::hyper::Body::from(json))?;

            Ok(response)
        }
    }

    impl_handle!(Delete);
}
