#[derive(Debug, Clone)]
pub struct Session<S> {
    user: String,
    broadcaster: tokio::sync::broadcast::Sender<types::Push>,
    service: S,
}

#[derive(Debug, Clone)]
pub struct Auth<S> {
    sessions: std::sync::Arc<Vec<Session<S>>>,
}

impl<S> Auth<S> {
    pub fn new(services: std::collections::HashMap<String, S>) -> Self {
        let mut sessions = Vec::with_capacity(services.len());
        for (user, service) in services {
            let (broadcaster, _) = tokio::sync::broadcast::channel::<types::Push>(16);
            let session = Session {
                user,
                broadcaster,
                service,
            };
            sessions.push(session);
        }
        sessions.sort_unstable_by(|a, b| a.user.cmp(&b.user));

        Self {
            sessions: std::sync::Arc::new(sessions),
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
        S: Clone,
        I: tower_service::Service<
            (hyper::Request<B>, Session<S>),
            Response = hyper::Response<Vec<u8>>,
        >,
        I::Future: Unpin,
        I::Error: Unpin,
    {
        type Response = I::Response;
        type Error = I::Error;
        type Future = Future<I::Future, I::Error>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
        fn call(&mut self, request: hyper::Request<B>) -> Self::Future {
            static X_USER: hyper::header::HeaderName =
                hyper::header::HeaderName::from_static("x-user");

            let Some(user_header) = request.headers().get(&X_USER) else {
                tracing::warn!(header = %X_USER, "Header is missing");
                return Future::Forbidden;
            };

            let user = match user_header.to_str() {
                Ok(user) => user,
                Err(error) => {
                    tracing::warn!(header = %X_USER, %error, "Header is not parseable as a String");
                    return Future::Forbidden;
                }
            };

            let Ok(index) = self
                .auth
                .sessions
                .binary_search_by(|u| u.user.as_str().cmp(user))
            else {
                tracing::warn!(%user, "User is not authorized");
                return Future::Forbidden;
            };

            // SAFETY: This was addressed above and the Vec is immutable
            let session = unsafe { self.auth.sessions.get_unchecked(index) };

            // TODO: Can I avoid this clone?
            Future::pass(self.inner.call((request, session.clone())))
        }
    }

    pub enum Future<F, E> {
        Forbidden,
        Pass(F, std::marker::PhantomData<E>),
    }

    impl<F, E> Future<F, E> {
        fn pass(future: F) -> Self {
            Self::Pass(future, std::marker::PhantomData)
        }
    }

    impl<F, E> std::future::Future for Future<F, E>
    where
        F: std::future::Future<Output = Result<hyper::Response<Vec<u8>>, E>> + Unpin,
        E: Unpin,
    {
        type Output = F::Output;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match self.get_mut() {
                Self::Forbidden => {
                    let mut response = hyper::Response::new(Vec::new());
                    *response.status_mut() = hyper::StatusCode::FORBIDDEN;
                    std::task::Poll::Ready(Ok(response))
                }
                Self::Pass(f, _) => std::pin::Pin::new(f).poll(cx),
            }
        }
    }
}
