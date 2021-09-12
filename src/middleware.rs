use crate::handler;
use crate::store;

type HandlerFuture = std::pin::Pin<Box<gotham::handler::HandlerFuture>>;

#[derive(Clone, gotham_derive::NewMiddleware)]
pub struct Log;

impl Log {
    fn request_info(
        state: &gotham::state::State,
    ) -> (String, &gotham::hyper::Method, &gotham::hyper::Uri, &str) {
        use gotham::hyper;
        use gotham::state::FromState;

        let ip = hyper::HeaderMap::borrow_from(state)
            .get(hyper::header::HeaderName::from_static("x-forwarded-for"))
            .and_then(|fwd| fwd.to_str().ok())
            .map_or_else(
                || {
                    gotham::state::client_addr(state)
                        .map_or_else(|| String::from("??"), |addr| addr.ip().to_string())
                },
                |fwd| format!("{} [p]", fwd),
            );

        // Request info
        let method = hyper::Method::borrow_from(state);
        let path = hyper::Uri::borrow_from(state);
        let length = hyper::HeaderMap::borrow_from(state)
            .get(hyper::header::CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok())
            .unwrap_or("");

        (ip, method, path, length)
    }

    fn log_level(error: &handler::Error) -> log::Level {
        use handler::Error;
        match error {
            Error::Store(store::Error::NotFound(_)) | Error::Deserialize(_) => log::Level::Info,
            Error::Store(store::Error::StoreFull) => log::Level::Warn,
            Error::FailedToAcquireLock | Error::Serialize(_) | Error::Http(_) | Error::Hyper(_) => {
                log::Level::Error
            }
        }
    }
}

impl gotham::middleware::Middleware for Log {
    fn call<Chain>(self, state: gotham::state::State, chain: Chain) -> HandlerFuture
    where
        Chain: FnOnce(gotham::state::State) -> HandlerFuture + Send + 'static,
    {
        Box::pin(async {
            chain(state)
                .await
                .map(|(state, response)| {
                    let (ip, method, path, length) = Self::request_info(&state);

                    // Response info
                    let status = response.status().as_u16();

                    // Log out
                    log::info!("{} {} - {} {} {}", status, ip, method, path, length);

                    (state, response)
                })
                .map_err(|(state, error)| {
                    let (ip, method, path, length) = Self::request_info(&state);

                    // Response info
                    let status = error.status().as_u16();

                    // Log out
                    if let Some(error) = error.downcast_cause_ref::<handler::Error>() {
                        log::log!(
                            Self::log_level(error),
                            "{} {} - {} {} {} [{}]",
                            status,
                            ip,
                            method,
                            path,
                            length,
                            error,
                        );
                    } else {
                        log::error!(
                            "{} {} - {} {} {} [Unknown error]",
                            status,
                            ip,
                            method,
                            path,
                            length
                        );
                    }

                    (state, error)
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

    pub fn get(
        &mut self,
    ) -> Result<
        std::sync::MutexGuard<'_, dyn store::Store>,
        std::sync::PoisonError<std::sync::MutexGuard<'_, dyn store::Store>>,
    > {
        self.0.lock()
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
