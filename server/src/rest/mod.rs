mod occurrence;
mod quick;
mod skull;

use crate::service::Service;

pub fn build() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::post(root))
        .nest("/skull", skull::build())
        .nest("/quick", quick::build())
        .nest("/occurrence", occurrence::build())
}

async fn root(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<types::Request>,
) -> (hyper::StatusCode, axum::Json<types::Response>) {
    handle(service, request).await
}

async fn handle(
    service: Service,
    request: types::Request,
) -> (hyper::StatusCode, axum::Json<types::Response>) {
    match service.handle(request).await.map_err(types::Error::from) {
        Ok(ok) => {
            let status = match ok {
                types::Payload::Created => hyper::StatusCode::CREATED,
                types::Payload::Updated | types::Payload::Deleted => hyper::StatusCode::NO_CONTENT,
                types::Payload::Skulls(_)
                | types::Payload::Quicks(_)
                | types::Payload::Occurrences(_) => hyper::StatusCode::OK,
            };
            (status, axum::Json(types::Response::Ok(ok)))
        }
        Err(error) => {
            let status = match error.kind {
                types::Kind::BadRequest => hyper::StatusCode::BAD_REQUEST,
                types::Kind::NotFound => hyper::StatusCode::NOT_FOUND,
                types::Kind::InternalError => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, axum::Json(types::Response::Error(error)))
        }
    }
}
