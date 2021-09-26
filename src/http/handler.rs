use super::error::Error;
use super::mapper;
use super::middleware;
use crate::store;

macro_rules! impl_handler {
    ($handler: ty, $param: tt) => {
        impl<$param> $handler {
            pub fn new() -> Self {
                Self(Default::default())
            }
        }

        impl<$param> Clone for $handler {
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }
        impl<$param> Copy for $handler {}

        impl<$param: store::CrudSelector> gotham::handler::Handler for $handler
        where
            $param: 'static + Send + Sync,
        {
            fn handle(
                self,
                mut state: gotham::state::State,
            ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
                Box::pin(async {
                    match Self::handle(&mut state).await {
                        Ok(r) => Ok((state, r)),
                        Err(e) => Err((state, e.into_handler_error())),
                    }
                })
            }
        }
    };
}

pub struct List<D>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> List<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let json = {
            let mut lock = middleware::Store::borrow_mut_from(state).get()?;
            let data = D::select(&mut *lock).list()?;
            serde_json::to_vec(&data)?
        };

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

impl_handler!(List<D>, D);

pub struct Create<D>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> Create<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let data = mapper::request::body(state).await?;

        let id = {
            let mut lock = middleware::Store::borrow_mut_from(state).get()?;
            D::select(&mut *lock).create(data)?
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::LOCATION, id)
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id::request_id(state),
            )
            .status(gotham::hyper::StatusCode::CREATED)
            .body(gotham::hyper::Body::empty())?;

        Ok(response)
    }
}

impl_handler!(Create<D>, D);

pub struct Read<D>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> Read<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::take_from(state).id;

        let json = {
            let mut lock = middleware::Store::borrow_mut_from(state).get()?;
            let data = D::select(&mut *lock).read(id)?;
            serde_json::to_vec(data)?
        };

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

impl_handler!(Read<D>, D);

pub struct Update<D>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> Update<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::take_from(state).id;
        let data = mapper::request::body(state).await?;

        let json = {
            let mut lock = middleware::Store::borrow_mut_from(state).get()?;
            let data = D::select(&mut *lock).update(id, data)?;
            serde_json::to_vec(&data)?
        };

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

impl_handler!(Update<D>, D);

pub struct Delete<D>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> Delete<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::take_from(state).id;

        let json = {
            let mut lock = middleware::Store::borrow_mut_from(state).get()?;
            let data = D::select(&mut *lock).delete(id)?;
            serde_json::to_vec(&data)?
        };

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

impl_handler!(Delete<D>, D);
