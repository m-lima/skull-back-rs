use types::{
    request::{
        occurrence::{query::Error, Create, Delete, Search, Update},
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
    SearchQuery(search): SearchQuery,
) -> (hyper::StatusCode, axum::Json<Response>) {
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

#[repr(transparent)]
struct SearchQuery(Search);

impl<S> axum::extract::FromRequestParts<S> for SearchQuery {
    type Rejection = SearchQueryRejection;

    fn from_request_parts<'parts, 'state, 'extractor>(
        parts: &'parts mut hyper::http::request::Parts,
        _: &'state S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'extractor>,
    >
    where
        'parts: 'extractor,
        'state: 'extractor,
        Self: 'extractor,
    {
        Box::pin(async move {
            Search::from_query(parts.uri.query().unwrap_or(""))
                .map(SearchQuery)
                .map_err(SearchQueryRejection)
        })
    }
}

#[repr(transparent)]
struct SearchQueryRejection(Error);

impl axum::response::IntoResponse for SearchQueryRejection {
    fn into_response(self) -> axum::response::Response {
        (hyper::StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
    }
}
