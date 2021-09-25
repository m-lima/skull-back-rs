use super::error;
use super::middleware;
use crate::store;

// #[derive(Copy, Clone, Debug)]
// pub enum ContentType {
//     Json,
// }

// pub struct Payload(Vec<u8>, ContentType);

// pub trait Serializer {
//     fn to_payload<T: serde::Serialize>(&self, payload: &T) -> Result<Payload, error::Error>;
// }

// struct JsonSerializer;

// impl Serializer for JsonSerializer {
//     fn to_payload<T: serde::Serialize>(&self, payload: &T) -> Result<Payload, error::Error> {
//         serde_json::to_vec(payload)
//             .map_err(error::Error::Serialize)
//             .map(|vec| Payload(vec, ContentType::Json))
//     }
// }

// TODO: Avoid the ownership of `Output`
#[derive(Clone)]
pub struct List<HandlerFunc, Output>(HandlerFunc)
where
    HandlerFunc: FnOnce(&mut dyn store::Store) -> Result<Output, error::Error>;

impl<HandlerFunc, Output> List<HandlerFunc, Output>
where
    HandlerFunc: FnOnce(&mut dyn store::Store) -> Result<Output, error::Error>,
    Output: serde::ser::Serialize,
{
    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
        use gotham::state::FromState;

        let data = (self.0)(&mut *middleware::Store::borrow_mut_from(state).get()?)?;
        let json = serde_json::to_vec(&data).map_err(error::Error::Serialize)?;

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

impl<HandlerFunc, Output> List<HandlerFunc, Output>
where
    HandlerFunc: FnOnce(&mut dyn store::Store) -> Result<Output, error::Error>,
    Output: serde::Serialize,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func)
    }

    async fn wrap(self, mut state: gotham::state::State) -> gotham::handler::HandlerResult {
        match self.handle(&mut state).await {
            Ok(r) => Ok((state, r)),
            Err(e) => Err((state, e.into_handler_error())),
        }
    }
}

impl<HandlerFunc, Output> gotham::handler::Handler for List<HandlerFunc, Output>
where
    HandlerFunc: FnOnce(&mut dyn store::Store) -> Result<Output, error::Error> + 'static + Send,
    Output: 'static + serde::Serialize,
{
    fn handle(
        self,
        state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(self.wrap(state))
    }
}

impl<HandlerFunc, Output> gotham::handler::NewHandler for List<HandlerFunc, Output>
where
    HandlerFunc: FnOnce(&mut dyn store::Store) -> Result<Output, error::Error>
        + 'static
        + Clone
        + Send
        + Sync
        + std::panic::RefUnwindSafe,
    Output: 'static + Clone + serde::Serialize,
{
    type Instance = Self;

    fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
        Ok(self.clone())
    }
}

// #[derive(Copy, Clone)]
// pub struct Create<HandlerFunc, Data>(HandlerFunc, std::marker::PhantomData<Data>)
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>, Data) -> Result<String, error::Error>;

// impl<HandlerFunc, Data> Create<HandlerFunc, Data>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>, Data) -> Result<String, error::Error>,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
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

// #[derive(Copy, Clone, skull_macro::Handler)]
// pub struct Read<HandlerFunc>(HandlerFunc)
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>;

// impl<HandlerFunc> Read<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
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

// #[derive(Copy, Clone, skull_macro::Handler)]
// pub struct Update<HandlerFunc>(HandlerFunc)
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>;

// impl<HandlerFunc> Update<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
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

// #[derive(Copy, Clone, skull_macro::Handler)]
// pub struct Delete<HandlerFunc>(HandlerFunc)
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>;

// impl<HandlerFunc> Delete<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
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

// impl_handle!(List<F>, F);

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
