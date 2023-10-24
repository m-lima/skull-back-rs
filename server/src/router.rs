use crate::auth::Session;

pub struct Router;

impl<B, S> tower_service::Service<(hyper::Request<B>, Session<S>)> for Router {
    type Response = ();
    type Error = ();
    type Future = Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, (request, session): (hyper::Request<B>, Session<S>)) -> Self::Future {
        let path = request.uri().path();
        let path = path.strip_suffix('/').unwrap_or(path);

        let mode = match path {
            "" => {
                if request.method() == hyper::Method::POST {
                    request.body()
                } else {
                    Future::MethodNotAllowed
                }
            }
            "skulll" => Mode::Binary,
            "quick" => Mode::Binary,
            "occurrence" => Mode::Binary,
            _ => Future::NotFound,
        };
    }
}

fn get_body<B>(request: hyper::Request<B>) -> types::Request
where
    B: hyper::body::HttpBody,
{
}

pub enum Future {
    Rest,
    Ws,
    MethodNotAllowed,
    NotFound,
    PayloadTooLarge,
}
