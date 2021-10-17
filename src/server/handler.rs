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

        impl<$param: store::Selector> gotham::handler::Handler for $handler
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

#[derive(Copy, Clone)]
pub struct LastModified;

impl LastModified {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let json = {
            let lock = middleware::Store::borrow_from(state).get()?;
            let data = lock.last_modified(user)?;
            serde_json::to_vec(&data)?
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id::request_id(state),
            )
            .header(gotham::hyper::header::CACHE_CONTROL, "no-cache")
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(json))?;

        Ok(response)
    }
}

impl gotham::handler::Handler for LastModified {
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

pub struct List<D>(std::marker::PhantomData<D>);

impl<D: store::Selector> List<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let json = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let data = D::select(&mut *lock).list(user)?;
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

impl<D: store::Selector> Create<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state).map(String::from)?;
        let data = mapper::request::Body::take_from(state).await?;

        let id = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            D::select(&mut *lock).create(&user, data)?
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

impl<D: store::Selector> Read<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let id = mapper::request::Id::borrow_from(state).id;

        let json = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let data = D::select(&mut *lock).read(user, id)?;
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

impl_handler!(Read<D>, D);

pub struct Update<D>(std::marker::PhantomData<D>);

impl<D: store::Selector> Update<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state).map(String::from)?;
        let id = mapper::request::Id::borrow_from(state).id;
        let data = mapper::request::Body::take_from(state).await?;

        let json = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let data = D::select(&mut *lock).update(&user, id, data)?;
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

impl<D: store::Selector> Delete<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let id = mapper::request::Id::borrow_from(state).id;

        let json = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let data = D::select(&mut *lock).delete(user, id)?;
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
