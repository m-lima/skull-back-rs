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
            D::read(store, user)?.last_modified()?
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
        let (last_modified, json) = {
            let store = middleware::Store::borrow_from(state).get();
            let crud = D::read(store, user)?;

            let data = crud.list(limit.limit)?;
            (crud.last_modified()?, serde_json::to_vec(&data)?)
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

        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;

        let (last_modified, id) = {
            let store = middleware::Store::borrow_from(state).get();
            let mut crud = D::write(store, user)?;

            let data = crud.create(data)?;
            (crud.last_modified()?, serde_json::to_vec(&data)?)
        };

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
            .body(gotham::hyper::Body::from(id))?;

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

        let (last_modified, json) = {
            let store = middleware::Store::borrow_from(state).get();
            let crud = D::read(store, user)?;

            let data = crud.read(id)?;
            (crud.last_modified()?, serde_json::to_vec(&data)?)
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

        let id = mapper::request::Id::borrow_from(state).id;
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let data = mapper::request::Body::take_from(state).await?;
        let user = mapper::request::User::borrow_from(state)?;

        let (last_modified, json) = {
            let store = middleware::Store::borrow_from(state).get();
            let mut crud = D::write(store, user)?;

            if crud.last_modified()? > unmodified_since {
                return Err(Error::OutOfSync);
            }

            let data = crud.update(id, data)?;
            (crud.last_modified()?, serde_json::to_vec(&data)?)
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
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;

        let (last_modified, json) = {
            let store = middleware::Store::borrow_from(state).get();
            let mut crud = D::write(store, user)?;

            if crud.last_modified()? > unmodified_since {
                return Err(Error::OutOfSync);
            }

            let data = crud.delete(id)?;
            (crud.last_modified()?, serde_json::to_vec(&data)?)
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
            .body(gotham::hyper::Body::from(json))?;

        Ok(response)
    }
}

impl_handler!(Delete<D>, D);
