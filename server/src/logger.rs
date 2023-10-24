pub struct Logger;

mod inner {
    use super::Logger;
    use crate::auth::Session;
    use crate::error::Error;

    impl<I> tower_layer::Layer<I> for Logger {
        type Service = Middleware<I>;

        fn layer(&self, inner: I) -> Self::Service {
            Middleware { inner }
        }
    }

    pub struct Middleware<I> {
        inner: I,
    }

    impl<B, S, I> tower_service::Service<(hyper::Request<B>, Session<S>)> for Middleware<I>
    where
        I: tower_service::Service<
            (hyper::Request<B>, Session<S>),
            Response = Result<types::Payload, Error>,
        >,
    {
        type Response = hyper::Response<Vec<u8>>;
        type Error = I::Error;
        type Future = Future<I::Future, I::Error>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
        fn call(&mut self, req: (hyper::Request<B>, Session<S>)) -> Self::Future {
            let start = std::time::Instant::now();

            Future {
                future: self.inner.call(req),
                start,
                _error: std::marker::PhantomData,
            }
        }
    }

    pub struct Future<F, E> {
        future: F,
        start: std::time::Instant,
        _error: std::marker::PhantomData<E>,
    }

    impl<F, E> std::future::Future for Future<F, E> {
        type Output = hyper::Response<Vec<u8>>;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            todo!()
        }
    }
}
