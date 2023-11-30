mod occurrence;
mod quick;
mod skull;

use crate::service::Service;

pub fn build() -> axum::Router {
    axum::Router::new()
        .nest("/skull", skull::build())
        .nest("/quick", quick::build())
        .nest("/occurrence", occurrence::build())
}

async fn handle(
    service: Service,
    request: types::Request,
) -> (hyper::StatusCode, axum::Json<types::Response>) {
    match service.handle(request).await {
        types::Response::Payload(payload) => {
            let status = match payload {
                types::Payload::Change(types::Change::Created) => hyper::StatusCode::CREATED,
                types::Payload::Change(types::Change::Updated | types::Change::Deleted) => {
                    hyper::StatusCode::NO_CONTENT
                }
                types::Payload::Skulls(_)
                | types::Payload::Quicks(_)
                | types::Payload::Occurrences(_) => hyper::StatusCode::OK,
            };
            (status, axum::Json(types::Response::Payload(payload)))
        }
        types::Response::Error(error) => {
            let status = match error.kind {
                types::Kind::BadRequest => hyper::StatusCode::BAD_REQUEST,
                types::Kind::NotFound => hyper::StatusCode::NOT_FOUND,
                types::Kind::InternalError => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, axum::Json(types::Response::Error(error)))
        }
    }
}
