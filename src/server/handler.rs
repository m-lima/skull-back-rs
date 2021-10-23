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
            let mut lock = middleware::Store::borrow_from(state).get()?;
            D::select(&mut *lock).last_modified(user)?
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id::request_id(state),
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

        let user = mapper::request::User::borrow_from(state)?;
        let (last_modified, json) = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let crud = D::select(&mut *lock);

            let data = crud.list(user)?;
            (crud.last_modified(user)?, serde_json::to_vec(&data)?)
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
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

        let (last_modified, id) = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let crud = D::select(&mut *lock);

            let data = crud.create(&user, data)?;
            (crud.last_modified(&user)?, serde_json::to_vec(&data)?)
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "text/plain")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
            .header(
                gotham::helpers::http::header::X_REQUEST_ID,
                gotham::state::request_id::request_id(state),
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
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let crud = D::select(&mut *lock);

            let data = crud.read(user, id)?;
            (crud.last_modified(user)?, serde_json::to_vec(&data)?)
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
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
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;
        let data = mapper::request::Body::take_from(state).await?;

        let (last_modified, json) = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let crud = D::select(&mut *lock);

            if crud.last_modified(&user)? > unmodified_since {
                return Err(Error::OutOfSync);
            }

            let data = crud.update(&user, id, data)?;
            (crud.last_modified(&user)?, serde_json::to_vec(&data)?)
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
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
        let unmodified_since = mapper::request::UnmodifiedSince::borrow_from(state)?;

        let (last_modified, json) = {
            let mut lock = middleware::Store::borrow_from(state).get()?;
            let crud = D::select(&mut *lock);

            if crud.last_modified(user)? > unmodified_since {
                return Err(Error::OutOfSync);
            }

            let data = crud.delete(user, id)?;
            (crud.last_modified(user)?, serde_json::to_vec(&data)?)
        };

        let response = gotham::hyper::Response::builder()
            .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
            .header(
                gotham::hyper::header::LAST_MODIFIED,
                mapper::time::serialize(&last_modified)?,
            )
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
