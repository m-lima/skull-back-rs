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

        match path {
            "" => root(&request),
            "/skull" => skull(&request),
            "/quick" => skull(&request),
            "/occurrence" => skull(&request),
            _ => Future::NotFound,
        }
    }
}

pub enum Future {
    Rest,
    Ws,
    RootMethodNotAllowed,
    WsMethodNotAllowed,
    RestMethodNotAllowed,
    NotFound,
    PayloadTooLarge,
}

fn root<B>(request: &hyper::Request<B>) -> Future {
    if method != hyper::Method::POST {
        return Future::RootMethodNotAllowed;
    }
}

fn skull<B>(request: &hyper::Request<B>) -> Future {
    let method = request.method();
    if method == hyper::Method::GET {
    } else if method == hyper::Method::POST {
    } else if method == hyper::Method::PATCH {
    } else if method == hyper::Method::DELETE {
    } else {
        Future::RestMethodNotAllowed
    }
}
