use super::error::Error;
use super::mapper;
use super::middleware;
use crate::store;

macro_rules! impl_handler {
    ($handler: ident) => {
        impl<D, S> $handler<D, S> {
            pub fn new() -> Self {
                Self(Default::default(), Default::default())
            }
        }

        impl<D, S> Clone for $handler<D, S> {
            fn clone(&self) -> Self {
                Self(self.0, self.1)
            }
        }
        impl<D, S> Copy for $handler<D, S> {}

        impl<D, S> gotham::handler::Handler for $handler<D, S>
        where
            D: 'static + store::Selector + Send + Sync,
            S: store::Store,
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

pub struct LastModified<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> LastModified<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let last_modified = {
            let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(LastModified);

pub struct List<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> List<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let limit = mapper::request::Limit::take_from(state);
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(List);

pub struct Create<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> Create<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(Create);

pub struct Read<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> Read<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let id = mapper::request::Id::borrow_from(state).id;
        let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(Read);

pub struct Update<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> Update<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::borrow_from(state).id;
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;
        let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(Update);

pub struct Delete<D, S>(std::marker::PhantomData<D>, std::marker::PhantomData<S>);

impl<D: store::Selector, S: store::Store> Delete<D, S> {
    async fn handle(
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, Error> {
        use gotham::state::FromState;

        let user = mapper::request::User::borrow_from(state)?;
        let id = mapper::request::Id::borrow_from(state).id;
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let store = middleware::Store::<S>::borrow_from(state).get();
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

impl_handler!(Delete);
