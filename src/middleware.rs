use crate::store;

type HandlerFuture = std::pin::Pin<Box<gotham::handler::HandlerFuture>>;

#[derive(Clone, gotham_derive::NewMiddleware)]
pub struct Log;

impl gotham::middleware::Middleware for Log {
    fn call<Chain>(self, state: gotham::state::State, chain: Chain) -> HandlerFuture
    where
        Chain: FnOnce(gotham::state::State) -> HandlerFuture + Send + 'static,
    {
        Box::pin(async {
            chain(state).await.map(|(state, response)| {
                {
                    use gotham::hyper;
                    use gotham::state::FromState;

                    let ip = hyper::HeaderMap::borrow_from(&state)
                        .get(hyper::header::HeaderName::from_static("x-forwarded-for"))
                        .and_then(|fwd| fwd.to_str().ok())
                        .map_or_else(
                            || {
                                gotham::state::client_addr(&state).map_or_else(
                                    || String::from("??"),
                                    |addr| addr.ip().to_string(),
                                )
                            },
                            |fwd| format!("{} [p]", fwd),
                        );

                    // Request info
                    let path = hyper::Uri::borrow_from(&state);
                    let method = hyper::Method::borrow_from(&state);
                    let length = hyper::HeaderMap::borrow_from(&state)
                        .get(hyper::header::CONTENT_LENGTH)
                        .and_then(|len| len.to_str().ok())
                        .unwrap_or("");

                    // Response info
                    let status = response.status().as_u16();

                    // Log out
                    log::info!("{} {} - {} {} {}", status, ip, method, path, length);
                }

                (state, response)
            })
        })
    }
}

#[derive(Clone, gotham_derive::StateData)]
pub struct Store(std::sync::Arc<std::sync::Mutex<dyn store::Store>>);

impl Store {
    pub fn new(store: impl store::Store) -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(store)))
    }

    pub fn get(&self) -> &std::sync::Arc<std::sync::Mutex<dyn store::Store>> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut std::sync::Arc<std::sync::Mutex<dyn store::Store>> {
        &mut self.0
    }
}

impl gotham::middleware::Middleware for Store {
    fn call<Chain>(
        self,
        mut state: gotham::state::State,
        chain: Chain,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>>
    where
        Chain: FnOnce(gotham::state::State) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>>,
    {
        state.put(self);
        chain(state)
    }
}

impl gotham::middleware::NewMiddleware for Store {
    type Instance = Self;

    fn new_middleware(&self) -> gotham::anyhow::Result<Self::Instance> {
        Ok(self.clone())
    }
}
