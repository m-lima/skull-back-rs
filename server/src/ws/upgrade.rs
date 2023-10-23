#[derive(Copy, Clone, Debug)]
pub struct Upgrade;

pub trait Server<S> {
    type Future: std::future::Future<Output = ()>;
    fn serve();
}

mod inner {
    use super::super::{listen, Mode};
    use super::Upgrade;
    use crate::auth::Session;

    impl<I> tower_layer::Layer<I> for Upgrade {
        type Service = Middleware<I>;

        fn layer(&self, inner: I) -> Self::Service {
            Middleware { inner }
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Error {
        InvalidVersion,
        MissingKey,
        NotUpgradable,
    }

    impl Error {
        fn into_response(self) -> hyper::Response<Vec<u8>> {
            let message = match self {
                Error::InvalidVersion => "Invalid value for sec-websocket-version",
                Error::MissingKey => "Missing value for sec-websocket-key",
                Error::NotUpgradable => "The connection is not upgradable",
            };

            tracing::info!("{message}");
            let mut response = hyper::Response::new(Vec::from(message.as_bytes()));
            *response.status_mut() = hyper::StatusCode::BAD_REQUEST;
            response
        }
    }

    #[derive(Debug, Clone)]
    pub struct Middleware<I> {
        inner: I,
    }

    impl<B, S, I> tower_service::Service<(hyper::Request<B>, Session<S>)> for Middleware<I>
    where
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
        fn call(&mut self, (request, session): (hyper::Request<B>, Session<S>)) -> Self::Future {
            if let Some(mode) = check_upgrade(&request) {
                upgrade(request, session, mode)
            } else {
                Future::pass(self.inner.call((request, session)))
            }
        }
    }

    fn check_upgrade<B>(request: &hyper::Request<B>) -> Option<Mode> {
        if request.method() != hyper::Method::GET {
            return None;
        }

        let path = request.uri().path();
        let path = path.strip_suffix('/').unwrap_or(path);

        let mode = match path {
            "/ws/text" => Mode::Text,
            "/ws/binary" => Mode::Binary,
            _ => return None,
        };

        if !has_header(request.headers(), hyper::header::CONNECTION, b"upgrade") {
            return None;
        }

        if !has_header(request.headers(), hyper::header::UPGRADE, b"websocket") {
            return None;
        }

        Some(mode)
    }

    fn upgrade<B, S, F, E>(
        mut request: hyper::Request<B>,
        session: Session<S>,
        mode: Mode,
    ) -> Future<F, E> {
        if !has_header(
            request.headers(),
            hyper::header::SEC_WEBSOCKET_VERSION,
            b"13",
        ) {
            return Future::Error(Error::InvalidVersion);
        }

        let Some(key) = request
            .headers()
            .get(hyper::header::SEC_WEBSOCKET_KEY)
            .map(Clone::clone)
        else {
            return Future::Error(Error::MissingKey);
        };

        let Some(on_upgrade) = request
            .extensions_mut()
            .remove::<hyper::upgrade::OnUpgrade>()
        else {
            return Future::Error(Error::NotUpgradable);
        };

        let protocol = request
            .headers()
            .get(hyper::header::SEC_WEBSOCKET_PROTOCOL)
            .map(Clone::clone);

        tokio::spawn(async move {
            let upgrade = match on_upgrade.await {
                Ok(upgraded) => upgraded,
                Err(error) => {
                    tracing::error!(%error, "Failed to upgrade websockets connection");
                    return;
                }
            };

            let socket = tokio_tungstenite::WebSocketStream::from_raw_socket(
                upgrade,
                tungstenite::protocol::Role::Server,
                None,
            )
            .await;

            listen(socket, session, mode).await;
        });
        Future::Upgrade(key, protocol)
    }

    fn has_header(
        headers: &hyper::HeaderMap,
        name: hyper::header::HeaderName,
        value: &[u8],
    ) -> bool {
        for header in headers.get_all(name) {
            if header
                .as_bytes()
                .split(|&h| h == b',')
                .any(|h| trim(h).eq_ignore_ascii_case(value))
            {
                return true;
            }
        }
        false
    }

    fn trim(bytes: &[u8]) -> &[u8] {
        let bytes = if let Some(start) = bytes.iter().position(|x| !x.is_ascii_whitespace()) {
            &bytes[start..]
        } else {
            b""
        };

        let bytes = if let Some(last) = bytes.iter().rposition(|x| !x.is_ascii_whitespace()) {
            &bytes[..=last]
        } else {
            b""
        };

        bytes
    }

    pub enum Future<F, E> {
        Upgrade(
            hyper::header::HeaderValue,
            Option<hyper::header::HeaderValue>,
        ),
        Error(Error),
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
                Self::Upgrade(key, protocol) => {
                    #[allow(clippy::declare_interior_mutable_const)]
                    const UPGRADE: hyper::header::HeaderValue =
                        hyper::header::HeaderValue::from_static("upgrade");
                    #[allow(clippy::declare_interior_mutable_const)]
                    const WEBSOCKET: hyper::header::HeaderValue =
                        hyper::header::HeaderValue::from_static("websocket");

                    let mut builder = hyper::Response::builder()
                        .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
                        .header(hyper::header::CONNECTION, UPGRADE)
                        .header(hyper::header::UPGRADE, WEBSOCKET)
                        .header(
                            hyper::header::SEC_WEBSOCKET_ACCEPT,
                            tungstenite::handshake::derive_accept_key(key.as_bytes()),
                        );

                    if let Some(protocol) = protocol.take() {
                        builder = builder.header(hyper::header::SEC_WEBSOCKET_PROTOCOL, protocol);
                    }

                    let response = builder.body(Vec::new()).unwrap();
                    std::task::Poll::Ready(Ok(response))
                }
                Self::Error(error) => std::task::Poll::Ready(Ok(error.into_response())),
                Self::Pass(f, _) => std::pin::Pin::new(f).poll(cx),
            }
        }
    }
}
