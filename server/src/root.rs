use crate::service::Service;

struct UserService {
    user: String,
    service: Service,
}

pub struct Root {
    users: Vec<UserService>,
}

impl Root {
    pub fn new(users: std::collections::HashMap<String, Service>) -> Self {
        let users = users
            .into_iter()
            .map(|(user, service)| UserService { user, service })
            .collect();
        Self { users }
    }
}

impl tower_service::Service<hyper::Request<Vec<u8>>> for Root {
    type Response = hyper::Response<Vec<u8>>;
    type Error = std::convert::Infallible;
    type Future = Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    fn call(&mut self, request: hyper::Request<Vec<u8>>) -> Self::Future {
        let start = std::time::Instant::now();
        let path = String::from(request.uri().path());
        let method = request.method().clone();
        let length = request.body().len();

        let Some(service) = auth::call(&request, &self.users) else {
            return Future {
                start,
                path,
                method,
                length,
                outcome: false,
            };
        };

        let route = router::call(request);

        return Future {
            start,
            path,
            method,
            length,
            outcome: true,
        };
    }
}

pub struct Future {
    start: std::time::Instant,
    path: String,
    method: hyper::Method,
    length: usize,
    outcome: bool,
}

impl std::future::Future for Future {
    type Output = Result<hyper::Response<Vec<u8>>, std::convert::Infallible>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let latency = this.start.elapsed();
        let method = this.method.as_str();
        let path = this.path.as_str();
        let length = this.length;
        tracing::info!(method, path, length, ?latency, "Ok {}", this.outcome);
        todo!()
    }
}

// TODO: Move to top-level
mod auth {
    use crate::root::UserService;

    type Result<'a> = std::result::Result<&'a UserService, Error>;

    pub enum Error {
        MissingHeader,
        InvalidHeader(hyper::header::ToStrError),
        Forbidden,
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    pub fn call<'a>(
        request: &hyper::Request<Vec<u8>>,
        users: &'a [UserService],
    ) -> Option<&'a UserService> {
        const X_USER: hyper::header::HeaderName = hyper::header::HeaderName::from_static("x-user");

        let Some(user_header) = request.headers().get(&X_USER) else {
            tracing::warn!(header = %X_USER, "Header is missing");
            return None;
        };

        let user = match user_header.to_str() {
            Ok(user) => user,
            Err(error) => {
                tracing::warn!(header = %X_USER, %error, "Header is not parseable as a String");
                return None;
            }
        };

        if let Ok(index) = users.binary_search_by(|u| u.user.as_str().cmp(user)) {
            // SAFETY: This was addressed above and the Vec is immutable
            unsafe { Some(users.get_unchecked(index)) }
        } else {
            tracing::warn!(%user, "User is not authorized");
            None
        }
    }
}

mod router {
    type Result<T = Route> = std::result::Result<T, Error>;

    pub enum Route {
        Rest(types::Request),
        Ws,
    }

    pub enum Error {
        NotFound,
        MethodNotPost,
        MethodNotCrud,
        ContentTypeMissing,
        WrongContentType,
        ContentLengthMissing,
        MalformedContentLength(hyper::header::ToStrError),
        ContentLengthNotInteger(std::num::ParseIntError),
        ContentTooLarge,
        BadRequest(serde_json::Error),
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
    pub fn call(request: hyper::Request<Vec<u8>>) -> Result {
        let path = request.uri().path();
        let path = path.strip_suffix('/').unwrap_or(path);

        match path {
            "" => root(request),
            "skull" => skull(request),
            "quick" => quick(request),
            "occurrence" => occurrence(request),
            // "/ws/text" => skull(&request),
            // "/ws/binary" => skull(&request),
            _ => Err(Error::NotFound),
        }
    }

    fn root(request: hyper::Request<Vec<u8>>) -> Result {
        if request.method() == hyper::Method::POST {
            read_body::<types::Request>(request).map(Route::Rest)
        } else {
            Err(Error::MethodNotPost)
        }
    }

    fn skull(request: hyper::Request<Vec<u8>>) -> Result {
        if request.method() == hyper::Method::GET {
            Ok(Route::Rest(types::Request::Skull(
                types::request::Skull::List,
            )))
        } else if request.method() == hyper::Method::POST {
            read_body(request)
                .map(types::request::Skull::Create)
                .map(types::Request::Skull)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::PATCH {
            read_body(request)
                .map(types::request::Skull::Update)
                .map(types::Request::Skull)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::DELETE {
            read_body(request)
                .map(types::request::Skull::Delete)
                .map(types::Request::Skull)
                .map(Route::Rest)
        } else {
            Err(Error::MethodNotCrud)
        }
    }

    fn quick(request: hyper::Request<Vec<u8>>) -> Result {
        if request.method() == hyper::Method::GET {
            Ok(Route::Rest(types::Request::Quick(
                types::request::Quick::List,
            )))
        } else if request.method() == hyper::Method::POST {
            read_body(request)
                .map(types::request::Quick::Create)
                .map(types::Request::Quick)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::PATCH {
            read_body(request)
                .map(types::request::Quick::Update)
                .map(types::Request::Quick)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::DELETE {
            read_body(request)
                .map(types::request::Quick::Delete)
                .map(types::Request::Quick)
                .map(Route::Rest)
        } else {
            Err(Error::MethodNotCrud)
        }
    }

    fn occurrence(request: hyper::Request<Vec<u8>>) -> Result {
        if request.method() == hyper::Method::GET {
            if let Some(query) = request.uri().query() {
                read_query(query)
                    .map(types::request::Occurrence::Search)
                    .map(types::Request::Occurrence)
                    .map(Route::Rest)
            } else {
                Ok(Route::Rest(types::Request::Occurrence(
                    types::request::Occurrence::List,
                )))
            }
        } else if request.method() == hyper::Method::POST {
            read_body(request)
                .map(types::request::Occurrence::Create)
                .map(types::Request::Occurrence)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::PATCH {
            read_body(request)
                .map(types::request::Occurrence::Update)
                .map(types::Request::Occurrence)
                .map(Route::Rest)
        } else if request.method() == hyper::Method::DELETE {
            read_body(request)
                .map(types::request::Occurrence::Delete)
                .map(types::Request::Occurrence)
                .map(Route::Rest)
        } else {
            Err(Error::MethodNotCrud)
        }
    }

    fn read_body<T: serde::de::DeserializeOwned>(request: hyper::Request<Vec<u8>>) -> Result<T> {
        let content_type = request
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .ok_or(Error::ContentLengthMissing)?;

        if content_type.as_bytes() != b"application/json" {
            return Err(Error::WrongContentType);
        }

        let content_length = request
            .headers()
            .get(hyper::header::CONTENT_LENGTH)
            .ok_or(Error::ContentLengthMissing)?;

        let content_length = content_length
            .to_str()
            .map_err(Error::MalformedContentLength)?;

        let content_length = content_length
            .parse::<usize>()
            .map_err(Error::ContentLengthNotInteger)?;

        if content_length > 2 * 1024 * 1024 {
            return Err(Error::ContentTooLarge);
        }

        // TODO: What about if it is too big to fit?

        serde_json::from_slice(&request.into_body()).map_err(Error::BadRequest)
    }

    fn read_query(query: &str) -> Result<types::request::occurrence::Search> {
        let mut search = types::request::occurrence::Search {
            skulls: None,
            start: None,
            end: None,
            limit: None,
        };

        let parts = query.split('&').map(|p| p.split_once('='));
        for part in parts {
            if let Some((key, value)) = part {
                match key {
                    "skulls" => {
                        search.skulls = serde_json::from_str(value).map_err(Error::BadRequest)?
                    }
                    "start" => {
                        search.start = serde_json::from_str(value).map_err(Error::BadRequest)?
                    }
                    "end" => search.end = serde_json::from_str(value).map_err(Error::BadRequest)?,
                    "limit" => {
                        search.limit = serde_json::from_str(value).map_err(Error::BadRequest)?
                    }
                    _ => (),
                }
            }
        }
        Ok(search)
    }
}

// mod auth {
//     use crate::service::Service;
//
//     #[derive(Debug, Clone)]
//     pub struct UserService {
//         user: String,
//         service: Service,
//     }
//
//     #[derive(Debug, Clone)]
//     pub struct Auth {
//         users: std::sync::Arc<Vec<UserService>>,
//     }
//
//     impl Auth {
//         pub fn new(services: std::collections::HashMap<String, Service>) -> Self {
//             let mut users = Vec::with_capacity(services.len());
//             for (user, service) in services {
//                 let (broadcaster, _) = tokio::sync::broadcast::channel::<types::Push>(16);
//                 let user = UserService { user, service };
//                 users.push(user);
//             }
//             users.sort_unstable_by(|a, b| a.user.cmp(&b.user));
//
//             Self {
//                 users: std::sync::Arc::new(user),
//             }
//         }
//     }
//
//     impl<B> tower_service::Service<hyper::Request<B>> for Auth {
//         type Response = router::Response;
//         type Error = I::Error;
//         type Future = Future<I::Future, I::Error>;
//
//         fn poll_ready(
//             &mut self,
//             cx: &mut std::task::Context<'_>,
//         ) -> std::task::Poll<Result<(), Self::Error>> {
//             self.inner.poll_ready(cx)
//         }
//
//         #[tracing::instrument(level = tracing::Level::DEBUG, skip_all)]
//         fn call(&mut self, request: hyper::Request<B>) -> Self::Future {
//             static X_USER: hyper::header::HeaderName =
//                 hyper::header::HeaderName::from_static("x-user");
//
//             let Some(user_header) = request.headers().get(&X_USER) else {
//                 tracing::warn!(header = %X_USER, "Header is missing");
//                 return Future::Forbidden;
//             };
//
//             let user = match user_header.to_str() {
//                 Ok(user) => user,
//                 Err(error) => {
//                     tracing::warn!(header = %X_USER, %error, "Header is not parseable as a String");
//                     return Future::Forbidden;
//                 }
//             };
//
//             let Ok(index) = self
//                 .auth
//                 .sessions
//                 .binary_search_by(|u| u.user.as_str().cmp(user))
//             else {
//                 tracing::warn!(%user, "User is not authorized");
//                 return Future::Forbidden;
//             };
//
//             // SAFETY: This was addressed above and the Vec is immutable
//             let session = unsafe { self.auth.sessions.get_unchecked(index) };
//
//             // TODO: Can I avoid this clone?
//             Future::pass(self.inner.call((request, session.clone())))
//         }
//     }
//
//     pub enum Future<F, E> {
//         Forbidden,
//         Pass(F, std::marker::PhantomData<E>),
//     }
//
//     impl<F, E> Future<F, E> {
//         fn pass(future: F) -> Self {
//             Self::Pass(future, std::marker::PhantomData)
//         }
//     }
//
//     impl<F, E> std::future::Future for Future<F, E>
//     where
//         F: std::future::Future<Output = Result<hyper::Response<Vec<u8>>, E>> + Unpin,
//         E: Unpin,
//     {
//         type Output = F::Output;
//
//         fn poll(
//             self: std::pin::Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//         ) -> std::task::Poll<Self::Output> {
//             match self.get_mut() {
//                 Self::Forbidden => {
//                     let mut response = hyper::Response::new(Vec::new());
//                     *response.status_mut() = hyper::StatusCode::FORBIDDEN;
//                     std::task::Poll::Ready(Ok(response))
//                 }
//                 Self::Pass(f, _) => std::pin::Pin::new(f).poll(cx),
//             }
//         }
//     }
//
//     mod router {
//         pub enum Response {}
//         pub struct Router {
//             user: String,
//             service: crate::service::Service,
//         }
//     }
// }
