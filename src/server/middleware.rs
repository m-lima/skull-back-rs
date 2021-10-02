use super::error::Error;
use crate::store;

type HandlerFuture = std::pin::Pin<Box<gotham::handler::HandlerFuture>>;

#[derive(Clone, gotham_derive::NewMiddleware)]
pub struct Log;

impl Log {
    #[inline]
    fn log_level(error: &Error) -> log::Level {
        use store::Error as StoreError;

        match error {
            Error::Store(StoreError::NotFound(_) | StoreError::NoSuchUser(_))
            | Error::Deserialize(_)
            | Error::PayloadTooLarge
            | Error::MissingUser
            | Error::BadHeader
            | Error::ContentLengthMissing => log::Level::Info,
            Error::ReadTimeout => log::Level::Warn,
            Error::Store(StoreError::StoreFull)
            | Error::FailedToAcquireLock
            | Error::Serialize(_)
            | Error::Http(_)
            | Error::Hyper(_) => log::Level::Error,
        }
    }

    #[inline]
    fn log_level_for(status: u16) -> log::Level {
        if status < 400 {
            log::Level::Info
        } else if status < 500 {
            log::Level::Warn
        } else {
            log::Level::Error
        }
    }

    #[inline]
    fn status_to_color(status: u16) -> colored::ColoredString {
        use colored::Colorize;
        if status < 200 {
            status.to_string().blue()
        } else if status < 400 {
            status.to_string().green()
        } else if status < 500 {
            status.to_string().yellow()
        } else if status < 600 {
            status.to_string().red()
        } else {
            status.to_string().white()
        }
    }

    fn log(
        state: &gotham::state::State,
        level: log::Level,
        status: u16,
        tail: &str,
        start: std::time::Instant,
    ) {
        use gotham::hyper;
        use gotham::state::FromState;

        let ip = hyper::HeaderMap::borrow_from(state)
            .get("x-forwarded-for")
            .and_then(|fwd| fwd.to_str().ok())
            .map_or_else(
                || {
                    gotham::state::client_addr(state)
                        .map_or_else(|| String::from("??"), |addr| addr.ip().to_string())
                },
                |fwd| format!("{} [p]", fwd),
            );

        let user = hyper::HeaderMap::borrow_from(state)
            .get("x-user")
            .and_then(|fwd| fwd.to_str().ok())
            .unwrap_or("UNKNOWN");

        let method = hyper::Method::borrow_from(state);
        let path = hyper::Uri::borrow_from(state);
        let request_length = hyper::HeaderMap::borrow_from(state)
            .get(hyper::header::CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok())
            .map_or_else(String::new, |len| format!(" {}b", len));

        // Log out
        log::log!(
            level,
            "{} {} {} {}{} - {}{} - {:?}",
            ip,
            user,
            method,
            path,
            request_length,
            Self::status_to_color(status),
            tail,
            start.elapsed()
        );
    }
}

impl gotham::middleware::Middleware for Log {
    fn call<Chain>(self, state: gotham::state::State, chain: Chain) -> HandlerFuture
    where
        Chain: FnOnce(gotham::state::State) -> HandlerFuture + Send + 'static,
    {
        Box::pin(async {
            let start = std::time::Instant::now();
            chain(state)
                .await
                .map(move |(state, response)| {
                    let status = response.status().as_u16();
                    let length = gotham::hyper::body::HttpBody::size_hint(response.body())
                        .exact()
                        .filter(|len| *len > 0)
                        .map_or_else(String::new, |len| format!(" {}b", len));

                    Self::log(&state, log::Level::Info, status, &length, start);

                    (state, response)
                })
                .map_err(|(state, error)| {
                    let status = error.status().as_u16();
                    let (level, error_message) = error.downcast_cause_ref::<Error>().map_or_else(
                        || (Self::log_level_for(status), " [Unknown error]".to_owned()),
                        |e| (Self::log_level(e), format!(" [{}]", e)),
                    );

                    Self::log(&state, level, status, &error_message, start);

                    (state, error)
                })
        })
    }
}

// TODO: Got a big ol lock here, for all users, all data types
#[derive(Clone, gotham_derive::StateData, gotham_derive::NewMiddleware)]
pub struct Store(std::sync::Arc<std::sync::Mutex<dyn store::Store>>);

impl Store {
    pub fn new(store: impl store::Store) -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(store)))
    }

    pub fn get(
        &self,
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

#[derive(Clone, gotham_derive::StateData, gotham_derive::NewMiddleware)]
pub struct Cors(gotham::hyper::header::HeaderValue);

impl Cors {
    pub fn new(cors: gotham::hyper::header::HeaderValue) -> Self {
        Self(cors)
    }
}

impl gotham::middleware::Middleware for Cors {
    fn call<Chain>(
        self,
        state: gotham::state::State,
        chain: Chain,
    ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>>
    where
        Chain: FnOnce(gotham::state::State) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>>
            + 'static
            + Send,
    {
        Box::pin(async {
            chain(state).await.map(|(state, mut response)| {
                let headers = response.headers_mut();
                headers.insert(gotham::hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, self.0);
                (state, response)
            })
        })
    }
}
