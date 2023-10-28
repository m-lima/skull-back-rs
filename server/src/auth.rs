#[derive(Debug, Clone)]
pub struct Session<S> {
    user: String,
    service: S,
}

impl<S> Session<S> {
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn service(&self) -> &S {
        &self.service
    }

    pub fn decompose(self) -> (String, S) {
        (self.user, self.service)
    }
}

impl<S> std::ops::Deref for Session<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl<S> std::ops::DerefMut for Session<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

#[derive(Debug, Clone)]
pub struct Auth<S> {
    services: std::sync::Arc<std::collections::HashMap<String, S>>,
}

impl<S> Auth<S> {
    pub fn wrap(services: std::collections::HashMap<String, S>) -> Self {
        Self {
            services: std::sync::Arc::new(services),
        }
    }
}

mod inner {
    use super::{Auth, Session};

    impl<S, I> tower_layer::Layer<I> for Auth<S>
    where
        S: Clone,
    {
        type Service = Middleware<S, I>;

        fn layer(&self, inner: I) -> Self::Service {
            Middleware {
                inner,
                auth: self.clone(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Middleware<S, I> {
        inner: I,
        auth: Auth<S>,
    }

    impl<B, S, I> tower_service::Service<hyper::Request<B>> for Middleware<S, I>
    where
        S: Clone + Send + Sync + 'static,
        I: tower_service::Service<hyper::Request<B>, Response = axum::response::Response>,
        I::Error: std::fmt::Display,
        I::Future: Unpin,
    {
        type Response = I::Response;
        type Error = I::Error;
        type Future = Future<I::Future>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, mut request: hyper::Request<B>) -> Self::Future {
            if let Some(session) = pre_auth(&request, &self.auth.services) {
                let span = tracing::span!(target: "layer", tracing::Level::DEBUG, "auth", user = %session.user);
                request.extensions_mut().insert(session);
                Future::Pass(self.inner.call(request), span)
            } else {
                Future::Forbidden
            }
        }
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, target = "layer", skip_all)]
    fn pre_auth<B, S>(
        request: &hyper::Request<B>,
        services: &std::collections::HashMap<String, S>,
    ) -> Option<Session<S>>
    where
        S: Clone,
    {
        let header = crate::X_USER;

        let Some(user_header) = request.headers().get(&header) else {
            tracing::warn!(%header, "Header is missing");
            return None;
        };

        let user = match user_header.to_str() {
            Ok(user) => user,
            Err(error) => {
                tracing::warn!(%header, %error, "Header is not parseable as a String");
                return None;
            }
        };

        let Some(service) = services.get(user) else {
            return None;
        };

        Some(Session {
            user: String::from(user),
            service: service.clone(),
        })
    }

    pub enum Future<F> {
        Forbidden,
        Pass(F, tracing::Span),
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
            match self.get_mut() {
                Self::Forbidden => {
                    let response =
                        axum::response::IntoResponse::into_response(hyper::StatusCode::FORBIDDEN);
                    std::task::Poll::Ready(Ok(response))
                }
                Self::Pass(f, span) => {
                    let _span = span.enter();
                    std::pin::Pin::new(f).poll(cx)
                }
            }
        }
    }
}
