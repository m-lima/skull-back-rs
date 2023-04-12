#![allow(warnings)]

use crate::options;
use crate::store;

pub fn routes(options: options::Options) -> anyhow::Result<axum::Router> {
    let router = match (options.db_path, options.store_path) {
        (Some(path), None) => route_store(store::in_db(path, options.users)?),
        (None, Some(path)) => route_store(store::in_file(path, options.users)?),
        (None, None) => route_store(store::in_memory(options.users)),
        (Some(_), Some(_)) => {
            unreachable!("Having both db and storage paths should be blocked by clap")
        }
    };

    let router = match (options.cors, options.web_path) {
        (Some(cors), None) => add_cors(router, cors),
        (None, Some(web_path)) => nest_api(router, web_path),
        (None, None) => router,
        (Some(_), Some(_)) => {
            unreachable!("Having both cors and web_path should be blocked by clap")
        }
    };

    Ok(router)
}

fn route_store<S: store::Store>(store: S) -> axum::Router {
    route_models().with_state(std::sync::Arc::new(store))
}

fn route_models<S: store::Store>() -> axum::Router<std::sync::Arc<S>> {
    fn route_model<S: store::Store, M: store::Model>(
        router: axum::Router<std::sync::Arc<S>>,
        _: std::marker::PhantomData<M>,
    ) -> axum::Router<std::sync::Arc<S>> {
        router.nest(
            M::name(),
            axum::Router::new()
                .route(
                    "/",
                    axum::routing::get(handler::list::<S, M>)
                        .post(handler::create::<S, M>)
                        .head(handler::head::<S, M>),
                )
                .route(
                    "/:id",
                    axum::routing::get(handler::read::<S, M>)
                        .put(handler::update::<S, M>)
                        .delete(handler::delete::<S, M>),
                ),
        )
    }

    let (m1, m2, m3) = store::MODELS;
    let mut router = axum::Router::new();
    router = route_model::<S, _>(router, m1);
    router = route_model::<S, _>(router, m2);
    route_model::<S, _>(router, m3)
}

fn add_cors(router: axum::Router, cors: axum::http::HeaderValue) -> axum::Router {
    use tower_http::cors;

    router.layer(
        cors::CorsLayer::new()
            .allow_origin(cors)
            .allow_methods(cors::AllowMethods::mirror_request())
            .allow_headers(cors::AllowHeaders::mirror_request()),
    )
}

fn nest_api(router: axum::Router, web_path: std::path::PathBuf) -> axum::Router {
    axum::Router::new()
        .route_service(
            "/",
            axum::routing::get_service(tower_http::services::ServeDir::new(web_path))
                .handle_error(|_| async { error::Error::NotFound }),
        )
        .nest("/api", router)
}

mod error {
    use crate::store;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("{0}")]
        Store(#[from] store::Error),
        #[error("Not found")]
        NotFound,
        #[error("Bad header")]
        BadHeader,
        #[error("Missing user header")]
        MissingUser,
        #[error("Client request is out of sync")]
        OutOfSync,
        #[error("Failed to deserialize timestamp: {0}")]
        TimeDeserialize(#[from] std::num::ParseIntError),
    }

    fn store_error_to_status_code(error: &store::Error) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match error {
            store::Error::NoSuchUser(_) => StatusCode::FORBIDDEN,
            store::Error::NotFound(_) => StatusCode::NOT_FOUND,
            store::Error::StoreFull => StatusCode::INSUFFICIENT_STORAGE,
            store::Error::Constraint | store::Error::Conflict => StatusCode::BAD_REQUEST,
            store::Error::Io(_)
            | store::Error::Serde(_)
            | store::Error::Lock
            | store::Error::Sql(_)
            | store::Error::BadMillis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    impl axum::response::IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            use axum::http::StatusCode;

            match self {
                Error::Store(ref err) => store_error_to_status_code(err),
                Error::NotFound => StatusCode::NOT_FOUND,
                Error::MissingUser => StatusCode::FORBIDDEN,
                Error::BadHeader | Self::TimeDeserialize(_) => StatusCode::BAD_REQUEST,
                Error::OutOfSync => StatusCode::PRECONDITION_FAILED,
            }
            .into_response()
        }
    }
}

mod handler {
    use axum::extract;

    use crate::store;

    use super::error;

    pub async fn head<S: store::Store, M: store::Model>(
        User(user): User,
        extract::State(store): extract::State<std::sync::Arc<S>>,
    ) -> axum::response::Response {
        use store::Crud;

        let crud = M::select(&*store, &user).unwrap();
        let last_modified = crud.last_modified().await.unwrap();
        todo!()
    }

    pub async fn list<S: store::Store, M: store::Model>(
        User(user): User,
        extract::State(store): extract::State<std::sync::Arc<S>>,
        extract::Query(Limit { limit }): extract::Query<Limit>,
    ) -> axum::response::Response {
        use store::Crud;

        let crud = M::select(&*store, &user).unwrap();
        let (body, last_modified) = crud.list(limit).await.unwrap();
        todo!()
    }

    pub async fn create<S: store::Store, M: store::Model>(
        User(user): User,
        extract::State(store): extract::State<std::sync::Arc<S>>,
        extract::Json(data): extract::Json<M>,
    ) -> axum::response::Response {
        todo!()
    }

    pub async fn read<S: store::Store, M: store::Model>(
        User(user): User,
        extract::State(store): extract::State<std::sync::Arc<S>>,
        extract::Path(id): extract::Path<store::Id>,
    ) -> axum::response::Response {
        todo!()
    }

    pub async fn update<S: store::Store, M: store::Model>(
        User(user): User,
        UnmodifiedSince(unmodified_since): UnmodifiedSince,
        extract::State(store): extract::State<std::sync::Arc<S>>,
        extract::Path(id): extract::Path<store::Id>,
        extract::Json(data): extract::Json<M>,
    ) -> axum::response::Response {
        todo!()
    }

    pub async fn delete<S: store::Store, M: store::Model>(
        User(user): User,
        extract::State(store): extract::State<std::sync::Arc<S>>,
        extract::Path(id): extract::Path<store::Id>,
    ) -> axum::response::Response {
        todo!()
    }

    #[derive(serde::Deserialize)]
    pub struct Limit {
        limit: Option<u32>,
    }

    pub struct User(String);

    #[async_trait::async_trait]
    impl<S: Send + Sync> extract::FromRequestParts<S> for User {
        type Rejection = error::Error;

        async fn from_request_parts(
            parts: &mut axum::http::request::Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            parts
                .headers
                .get("X-User")
                .ok_or(error::Error::MissingUser)?
                .to_str()
                .map_err(|_| error::Error::BadHeader)
                .map(String::from)
                .map(Self)
        }
    }

    pub struct UnmodifiedSince(std::time::SystemTime);

    #[async_trait::async_trait]
    impl<S: Send + Sync> extract::FromRequestParts<S> for UnmodifiedSince {
        type Rejection = error::Error;

        async fn from_request_parts(
            parts: &mut axum::http::request::Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            parts
                .headers
                .get(axum::http::header::IF_UNMODIFIED_SINCE)
                .ok_or(error::Error::OutOfSync)?
                .to_str()
                .map_err(|_| error::Error::BadHeader)?
                .parse::<u64>()
                .map_err(error::Error::TimeDeserialize)
                .map(std::time::Duration::from_millis)
                .and_then(|millis| {
                    std::time::UNIX_EPOCH
                        .checked_add(millis)
                        .ok_or(error::Error::BadHeader)
                })
                .map(Self)
        }
    }
}
