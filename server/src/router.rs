pub fn build() -> axum::Router {
    axum::Router::new().nest("/", rest::build())
}

mod rest {
    use types::Request;

    use crate::service::Service;

    pub fn build() -> axum::Router {
        axum::Router::new()
        // .route("/", axum::routing::post(handle))
        // .nest("/skull", skull::build())
        // .nest("/quick", quick::build())
        // .nest("/occurrence", occurrence::build())
    }

    // async fn handle(
    //     axum::Extension(service): axum::Extension<Service>,
    //     axum::Json(request): axum::Json<Request>,
    // ) -> (hyper::StatusCode, axum::Json<Response>) {
    //     let response = service.handle(request).await;
    //     // (status_for(&response), axum::Json(response))
    //     todo!()
    // }
    //
    // mod skull {
    //     pub fn build() -> axum::Router {
    //         axum::Router::new()
    //             .route("/", axum::routing::get(get))
    //             .route("/", axum::routing::post(post))
    //             .route("/", axum::routing::put(put))
    //             .route("/", axum::routing::delete(delete))
    //     }
    //
    //     async fn get(
    //         axum::Extension(service): axum::Extension<Service>,
    //     ) -> Result<axum::Json<Vec<Task>>> {
    //         service.handle(types::request::Skull::List).await;
    //         todo!()
    //     }
    //
    //     async fn post(
    //         axum::Extension(service): axum::Extension<Service>,
    //         axum::Json(payload): axum::Json<Create>,
    //     ) -> Result<axum::response::Response> {
    //         let id = service.tasks().create(payload).await?;
    //         Ok(axum::response::IntoResponse::into_response((
    //             axum::http::StatusCode::CREATED,
    //             format!("{id}"),
    //         )))
    //     }
    //
    //     async fn patch(
    //         axum::Extension(service): axum::Extension<Service>,
    //         axum::Json(payload): axum::Json<Update>,
    //     ) -> Result<axum::http::StatusCode> {
    //         service.tasks().update(payload).await?;
    //         Ok(axum::http::StatusCode::NO_CONTENT)
    //     }
    //
    //     async fn delete(
    //         axum::Extension(service): axum::Extension<Service>,
    //         axum::Json(payload): axum::Json<Delete>,
    //     ) -> Result<axum::http::StatusCode> {
    //         service.tasks().delete(payload).await?;
    //         Ok(axum::http::StatusCode::NO_CONTENT)
    //     }
    // }
}
