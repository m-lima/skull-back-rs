use super::mapper;
use super::middleware;
use crate::store;

// TODO: Remove once try-blocks lands https://github.com/rust-lang/rust/issues/31436
macro_rules! impl_handle {
    ($name:ty, $F:tt) => {
        impl<F> $name
        where
            F: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, Error>,
        {
            pub fn new(handler_func: F) -> Self {
                Self(handler_func)
            }

            async fn wrap(self, mut state: gotham::state::State) -> gotham::handler::HandlerResult {
                match self.handle(&mut state).await {
                    Ok(r) => Ok((state, r)),
                    Err(e) => Err((state, e.into_handler_error())),
                }
            }
        }

        impl<F> gotham::handler::Handler for $name
        where
            F: 'static
                + Send
                + Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, Error>,
        {
            fn handle(
                self,
                state: gotham::state::State,
            ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
                Box::pin(self.wrap(state))
            }
        }

        impl<F> gotham::handler::NewHandler for $name
        where
            F: 'static
                + Copy
                + Send
                + Sync
                + std::panic::RefUnwindSafe
                + Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, Error>,
        {
            type Instance = Self;

            fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
                Ok(*self)
            }
        }
    };
}

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

    fn into_handler_error(self) -> gotham::handler::HandlerError {
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

#[derive(Copy, Clone)]
pub struct List<HandlerFunc>(HandlerFunc);

impl<HandlerFunc> List<HandlerFunc>
where
    HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, Error>,
{
    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let json = (self.0)(middleware::Store::borrow_mut_from(state).get()?)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(json))?;

        Ok(response)
    }
}

impl_handle!(List<F>, F);

// #[derive(Copy, Clone)]
// pub struct Create<HandlerFunc>(HandlerFunc);

// impl<HandlerFunc, Data> Create<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>, Data) -> Result<store::Id, Error>,
//     Data: store::Data,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
//         use gotham::state::FromState;

//         let json = (self.0)(middleware::Store::borrow_mut_from(state).get()?)?;

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::OK)
//             .body(gotham::hyper::Body::from(json))?;

//         Ok(response)
//     }
// }

// impl_handle!(Create<F>, F);

// #[derive(Copy, Clone)]
// pub struct Create;

// impl Create {
//     async fn handle(
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
//         use gotham::state::FromState;

//         let skull = mapper::request::body(state).await?;

//         let id = {
//             let mut store = middleware::Store::borrow_mut_from(state).get()?;
//             store.skull().create(skull)?
//         };

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::LOCATION, id)
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::CREATED)
//             .body(gotham::hyper::Body::empty())?;

//         Ok(response)
//     }
// }

// impl_handle!(Create);

// #[derive(Copy, Clone)]
// pub struct Read;

// impl Read {
//     async fn handle(
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
//         use gotham::state::FromState;

//         let id = mapper::request::Id::take_from(state).id;

//         let json = {
//             let mut store = middleware::Store::borrow_mut_from(state).get()?;
//             let skull = store.skull().read(id)?;
//             serde_json::to_string(&skull).map_err(Error::Serialize)?
//         };

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::OK)
//             .body(gotham::hyper::Body::from(json))?;

//         Ok(response)
//     }
// }

// impl_handle!(Read);

// #[derive(Copy, Clone)]
// pub struct Update;

// impl Update {
//     async fn handle(
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
//         use gotham::state::FromState;

//         let skull = mapper::request::body(state).await?;
//         let id = mapper::request::Id::take_from(state).id;

//         let json = {
//             let mut store = middleware::Store::borrow_mut_from(state).get()?;
//             let skull = store.skull().update(id, skull)?;
//             serde_json::to_string(&skull).map_err(Error::Serialize)?
//         };

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::OK)
//             .body(gotham::hyper::Body::from(json))?;

//         Ok(response)
//     }
// }

// impl_handle!(Update);

// #[derive(Copy, Clone)]
// pub struct Delete;

// impl Delete {
//     async fn handle(
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
//         use gotham::state::FromState;

//         let id = mapper::request::Id::take_from(state).id;

//         let json = {
//             let mut store = middleware::Store::borrow_mut_from(state).get()?;
//             let skull = store.skull().delete(id)?;
//             serde_json::to_string(&skull).map_err(Error::Serialize)?
//         };

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::OK)
//             .body(gotham::hyper::Body::from(json))?;

//         Ok(response)
//     }
// }

// impl_handle!(Delete);
