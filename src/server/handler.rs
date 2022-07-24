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

pub struct LastModified<D>(std::marker::PhantomData<D>);

impl<D: store::Selector> LastModified<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let last_modified = {
            let store = middleware::Store::borrow_from(state).get();
            D::select(store, user)?.last_modified().await?
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::empty())?;

        Ok(response)
    }
}

impl_handler!(LastModified<D>, D);

pub struct List<D>(std::marker::PhantomData<D>);

impl<D: store::Selector> List<D> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let limit = mapper::request::Limit::take_from(state);
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::borrow_from(state).get();
        let crud = D::select(store, user)?;

        let (body, last_modified) = crud.list(limit.limit).await?;
        let body = serde_json::to_vec(&body)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(body))?;

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

        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::borrow_from(state).get();
        let crud = D::select(store, user)?;

        let (body, last_modified) = crud.create(data).await?;
        let body = serde_json::to_vec(&body)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "text/plain")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::CREATED)
            .body(gotham::hyper::Body::from(body))?;

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
        let store = middleware::Store::borrow_from(state).get();
        let crud = D::select(store, user)?;

        let (body, last_modified) = crud.read(id).await?;
        let body = serde_json::to_vec(&body)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(body))?;

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

        let id = mapper::request::Id::borrow_from(state).id;
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::borrow_from(state).get();
        let crud = D::select(store, user)?;

        // TODO:
        // 1 - Move this responsibility to Crud
        // 2 - Update unit tests to cover this
        // -- so far no API breaking changes --
        // 3 - Receive a full object to ensure request is what it is
        // 4 - Update front-end to match
        // -- so far smaller effort --
        // 5 - Create websockets to push modifications
        if crud.last_modified().await? > unmodified_since {
            return Err(Error::OutOfSync);
        }

        let (body, last_modified) = crud.update(id, data).await?;
        let body = serde_json::to_vec(&body)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(body))?;

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
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let store = middleware::Store::borrow_from(state).get();
        let crud = D::select(store, user)?;

        if crud.last_modified().await? > unmodified_since {
            return Err(Error::OutOfSync);
        }

        let (body, last_modified) = crud.delete(id).await?;
        let body = serde_json::to_vec(&body)?;

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified),
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id(state),
            )
            .status(gotham::hyper::StatusCode::OK)
            .body(gotham::hyper::Body::from(body))?;

        Ok(response)
    }
}

impl_handler!(Delete<D>, D);
