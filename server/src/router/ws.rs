use crate::{
    service::Service,
    ws::{Mode, Socket},
};

pub fn build() -> axum::Router {
    axum::Router::new()
        .route("/text", axum::routing::get(upgrade::<String>))
        .route("/binary", axum::routing::get(upgrade::<Vec<u8>>))
}

// Allow(clippy::unused_async): To match axum's requirement
#[allow(clippy::unused_async)]
async fn upgrade<T: Mode>(
    upgrade: axum::extract::WebSocketUpgrade,
    axum::Extension(service): axum::Extension<Service>,
) -> axum::response::Response {
    upgrade.on_upgrade(|socket| Socket::<T>::new(socket, service).serve())
}
