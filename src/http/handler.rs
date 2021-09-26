use super::error;
use super::mapper;
use super::middleware;
use crate::store;

// List
pub struct List<HandlerFunc, Output>(HandlerFunc)
where
    HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error>;

impl<HandlerFunc, Output> List<HandlerFunc, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error>,
    Output: serde::ser::Serialize,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func)
    }

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

impl<HandlerFunc, Output> Clone for List<HandlerFunc, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error> + Copy,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<HandlerFunc, Output> Copy for List<HandlerFunc, Output> where
    HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error> + Copy
{
}

impl<HandlerFunc, Output> gotham::handler::Handler for List<HandlerFunc, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error> + 'static + Send,
    Output: serde::Serialize + 'static,
{
    fn handle(
        self,
        mut state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(async {
            match self.handle(&mut state).await {
                Ok(r) => Ok((state, r)),
                Err(e) => Err((state, e.into_handler_error())),
            }
        })
    }
}

// Create
pub struct Create<HandlerFunc, Data>(HandlerFunc, std::marker::PhantomData<Data>)
where
    HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
    Data: store::Data;

impl<HandlerFunc, Data> Create<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
    Data: store::Data,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func, std::marker::PhantomData::default())
    }

    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
        use gotham::state::FromState;

        let body = mapper::request::body(state).await?;

        let id = (self.0)(&mut *middleware::Store::borrow_mut_from(state).get()?, body)?;

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

impl<HandlerFunc, Data> Clone for Create<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error> + Copy,
    Data: store::Data,
{
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<HandlerFunc, Data> Copy for Create<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error> + Copy,
    Data: store::Data,
{
}

impl<HandlerFunc, Data> gotham::handler::Handler for Create<HandlerFunc, Data>
where
    HandlerFunc:
        Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error> + 'static + Send,
    Data: store::Data + 'static + Send,
{
    fn handle(
        self,
        state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(async {
            match self.handle(&mut state).await {
                Ok(r) => Ok((state, r)),
                Err(e) => Err((state, e.into_handler_error())),
            }
        })
    }
}

// Read
pub struct Read<HandlerFunc, Data>(HandlerFunc)
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error>,
    Data: store::Data;

impl<HandlerFunc, Data> Read<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error>,
    Data: store::Data,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func)
    }

    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::take_from(state).id;

        let data = (self.0)(&mut *middleware::Store::borrow_mut_from(state).get()?, id)?;
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

impl<HandlerFunc, Data> Clone for Read<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + Copy,
    Data: store::Data,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<HandlerFunc, Data> Copy for Read<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + Copy,
    Data: store::Data,
{
}

impl<HandlerFunc, Data> gotham::handler::Handler for Read<HandlerFunc, Data>
where
    HandlerFunc:
        Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + 'static + Send,
    Data: store::Data + 'static + Send,
{
    fn handle(
        self,
        state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(async {
            match self.handle(&mut state).await {
                Ok(r) => Ok((state, r)),
                Err(e) => Err((state, e.into_handler_error())),
            }
        })
    }
}

// Update
pub struct Update<HandlerFunc, Data, Output>(HandlerFunc, std::marker::PhantomData<Data>)
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id, Data) -> Result<Output, error::Error>,
    Data: store::Data;

impl<HandlerFunc, Data, Output> Update<HandlerFunc, Data, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id, Data) -> Result<Output, error::Error>,
    Data: store::Data,
    Output: serde::ser::Serialize,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func, std::marker::PhantomData::default())
    }

    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
        use gotham::state::FromState;

        let body = mapper::request::body(state).await?;
        let id = mapper::request::Id::take_from(state).id;

        let data = (self.0)(
            &mut *middleware::Store::borrow_mut_from(state).get()?,
            id,
            body,
        )?;
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

impl<HandlerFunc, Data, Output> Clone for Update<HandlerFunc, Data, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id, Data) -> Result<Output, error::Error> + Copy,
    Data: store::Data,
{
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<HandlerFunc, Data, Output> Copy for Update<HandlerFunc, Data, Output>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id, Data) -> Result<Output, error::Error> + Copy,
    Data: store::Data,
{
}

impl<HandlerFunc, Data, Output> gotham::handler::Handler for Update<HandlerFunc, Data, Output>
where
    HandlerFunc:
        Fn(&mut dyn store::Store, store::Id, Data) -> Result<Output, error::Error> + 'static + Send,
    Data: store::Data + 'static + Send,
    Output: 'static + serde::Serialize,
{
    fn handle(
        self,
        state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(async {
            match self.handle(&mut state).await {
                Ok(r) => Ok((state, r)),
                Err(e) => Err((state, e.into_handler_error())),
            }
        })
    }
}

// Delete
pub struct Delete<HandlerFunc, Data>(HandlerFunc)
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error>,
    Data: store::Data;

impl<HandlerFunc, Data> Delete<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error>,
    Data: store::Data,
{
    pub fn new(handler_func: HandlerFunc) -> Self {
        Self(handler_func)
    }

    async fn handle(
        self,
        state: &mut gotham::state::State,
    ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
        use gotham::state::FromState;

        let id = mapper::request::Id::take_from(state).id;

        let data = (self.0)(&mut *middleware::Store::borrow_mut_from(state).get()?, id)?;
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

impl<HandlerFunc, Data> Clone for Delete<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + Copy,
    Data: store::Data,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<HandlerFunc, Data> Copy for Delete<HandlerFunc, Data>
where
    HandlerFunc: Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + Copy,
    Data: store::Data,
{
}

impl<HandlerFunc, Data> gotham::handler::Handler for Delete<HandlerFunc, Data>
where
    HandlerFunc:
        Fn(&mut dyn store::Store, store::Id) -> Result<Data, error::Error> + 'static + Send,
    Data: store::Data + 'static + Send,
{
    fn handle(
        self,
        state: gotham::state::State,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
        Box::pin(async {
            match self.handle(&mut state).await {
                Ok(r) => Ok((state, r)),
                Err(e) => Err((state, e.into_handler_error())),
            }
        })
    }
}
