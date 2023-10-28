mod label;
mod task;

use crate::service::Service;
use types::{
    request::Request,
    response::{Kind, Response},
};

pub fn build() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::post(handle))
        .nest("/task", task::build())
        .nest("/label", label::build())
}

async fn handle(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(payload): axum::Json<Request>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    let response = service.handle(payload).await;
    (status_for(&response), axum::Json(response))
}
