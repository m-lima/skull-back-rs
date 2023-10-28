#[derive(Debug, Copy, Clone)]
pub struct Logger;

#[derive(Debug, Copy, Clone)]
pub struct Handled(bool);

mod inner {
    use super::{Handled, Logger};
    use crate::X_USER;

    impl<S> tower_layer::Layer<S> for Logger {
        type Service = Middleware<S>;

        fn layer(&self, inner: S) -> Self::Service {
            Middleware { inner }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Middleware<S> {
        inner: S,
    }

    impl<B, S> tower_service::Service<hyper::Request<B>> for Middleware<S>
    where
        S: tower_service::Service<hyper::Request<B>, Response = axum::response::Response>,
        S::Error: std::fmt::Display,
        S::Future: Unpin,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Future<S::Future>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: hyper::Request<B>) -> Self::Future {
            let start = std::time::Instant::now();
            let method = String::from(request.method().as_str());
            let path = String::from(request.uri().path());
            let length = get_length(request.headers());
            let user = request
                .headers()
                .get(X_USER)
                .and_then(|l| l.to_str().ok())
                .map(String::from);

            let span = tracing::span!(
                target: "layer",
                tracing::Level::DEBUG,
                "request",
                %method,
                %path
            );

            Future {
                span,
                start,
                method,
                path,
                user,
                length,
                future: self.inner.call(request),
            }
        }
    }

    pub struct Future<F> {
        span: tracing::Span,
        start: std::time::Instant,
        method: String,
        path: String,
        user: Option<String>,
        length: Option<usize>,
        future: F,
    }

    impl<F, E> std::future::Future for Future<F>
    where
        F: std::future::Future<Output = Result<axum::response::Response, E>> + Unpin,
        E: std::fmt::Display,
    {
        type Output = F::Output;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let this = self.get_mut();
            let _span = this.span.enter();
            let future = &mut this.future;

            let output = std::task::ready!(std::pin::Pin::new(future).poll(cx));

            match output {
                Ok(response) => {
                    if response.extensions().get::<Handled>().is_none() {
                        log_ok(this, &response);
                    }
                    std::task::Poll::Ready(Ok(response))
                }
                Err(error) => {
                    log_err(this, &error);
                    std::task::Poll::Ready(Err(error))
                }
            }
        }
    }

    fn log_ok<F>(future: &Future<F>, response: &axum::response::Response) {
        let latency = future.start.elapsed();
        let length = get_length(response.headers());
        let (status, spacer, reason) = get_status_tuple(response.status());

        macro_rules! log {
            ($level: expr) => {
                match (future.user.as_ref(), future.length, length) {
                    (Some(user), Some(incoming), Some(outgoing)) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            %user,
                            incoming,
                            outgoing,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (Some(user), None, Some(outgoing)) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            %user,
                            outgoing,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (None, Some(incoming), Some(outgoing)) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            incoming,
                            outgoing,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (None, None, Some(outgoing)) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            outgoing,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (Some(user), Some(incoming), None) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            %user,
                            incoming,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (Some(user), None, None) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            %user,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (None, Some(incoming), None) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            incoming,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                    (None, None, None) => {
                        tracing::event!(
                            $level,
                            method = %future.method,
                            path = %future.path,
                            ?latency,
                            "{status}{spacer}{reason}"
                        );
                    }
                }
            };
        }

        match status {
            0..=399 => log!(tracing::Level::INFO),
            400..=499 => log!(tracing::Level::WARN),
            500.. => log!(tracing::Level::ERROR),
        }
    }

    fn log_err<F, E: std::fmt::Display>(future: &Future<F>, error: &E) {
        match (future.user.as_ref(), future.length) {
            (Some(user), Some(incoming)) => {
                tracing::error!(method = future.method, path = future.path, user, incoming, %error, "Unexpected error while serving request");
            }
            (Some(user), None) => {
                tracing::error!(method = future.method, path = future.path, user, %error, "Unexpected error while serving request");
            }
            (None, Some(incoming)) => {
                tracing::error!(method = future.method, path = future.path, incoming, %error, "Unexpected error while serving request");
            }
            (None, None) => {
                tracing::error!(method = future.method, path = future.path, %error, "Unexpected error while serving request");
            }
        }
    }

    fn get_status_tuple(status: hyper::StatusCode) -> (u16, &'static str, &'static str) {
        match status.canonical_reason() {
            Some(reason) => (status.as_u16(), " ", reason),
            None => (status.as_u16(), "", ""),
        }
    }

    fn get_length(headers: &hyper::header::HeaderMap) -> Option<usize> {
        headers
            .get(hyper::header::CONTENT_LENGTH)
            .and_then(|l| l.to_str().ok())
            .and_then(|l| l.parse().ok())
            .filter(|l| *l > 0)
    }
}
