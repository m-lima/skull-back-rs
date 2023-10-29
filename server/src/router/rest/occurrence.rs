use types::{
    request::{
        occurrence::{Create, Delete, Search, Update},
        Occurrence,
    },
    Response,
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
    axum::extract::Query(search): axum::extract::Query<Search>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    // TODO: Looks like the query is mandatory
    if search.skulls.is_none()
        && search.start.is_none()
        && search.end.is_none()
        && search.limit.is_none()
    {
        super::handle(service, types::Request::Occurrence(Occurrence::List)).await
    } else {
        super::handle(
            service,
            types::Request::Occurrence(Occurrence::Search(search)),
        )
        .await
    }
}

async fn post(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Create>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(
        service,
        types::Request::Occurrence(Occurrence::Create(request)),
    )
    .await
}

async fn patch(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Update>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(
        service,
        types::Request::Occurrence(Occurrence::Update(request)),
    )
    .await
}

async fn delete(
    axum::Extension(service): axum::Extension<Service>,
    axum::Json(request): axum::Json<Delete>,
) -> (hyper::StatusCode, axum::Json<Response>) {
    super::handle(
        service,
        types::Request::Occurrence(Occurrence::Delete(request)),
    )
    .await
}
