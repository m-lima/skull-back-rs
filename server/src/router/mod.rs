mod rest;
mod ws;

pub fn build() -> axum::Router {
    rest::build().nest("/ws", ws::build())
}
