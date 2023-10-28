mod rest;
mod ws;

pub fn build() -> axum::Router {
    axum::Router::new()
        .nest("/", rest::build())
        .nest("/ws", ws::build())
}
