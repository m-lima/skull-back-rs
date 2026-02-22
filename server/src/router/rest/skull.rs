use types::{
    Response,
    request::{
        Skull,
        skull::{Create, Delete, Update},
    },
};

use crate::service::Service;

pub fn build() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(get))
        .route("/", axum::routing::post(post))
        .route("/", axum::routing::patch(patch))
        .route("/", axum::routing::delete(delete))
}

async fn get(
    axum::Extension(service): axum::Extension<Service>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(service, types::Request::Skull(Skull::List)).await
}

async fn post(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Create>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(service, types::Request::Skull(Skull::Create(request))).await
}

async fn patch(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Update>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(service, types::Request::Skull(Skull::Update(request))).await
}

async fn delete(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Delete>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(service, types::Request::Skull(Skull::Delete(request))).await
}
