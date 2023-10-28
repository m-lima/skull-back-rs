// macro_rules! pass {
//     ($flow_control: expr) => {
//         match $flow_control {
//             FlowControl::Pass(value) => value,
//             FlowControl::Break => break,
//             FlowControl::Continue => continue,
//         }
//     };
// }
//
// mod socket;
//
// use types::response::Push;

pub fn build() -> axum::Router {
    axum::Router::new()
    // .route("/text", axum::routing::get(upgrade::<String>))
    // .route("/binary", axum::routing::get(upgrade::<Vec<u8>>))
}

// // Allow(clippy::unused_async): To match axum's requirement
// #[allow(clippy::unused_async)]
// async fn upgrade<T: socket::Payload>(
//     upgrade: axum::extract::WebSocketUpgrade,
//     axum::Extension(service): axum::Extension<Service>,
// ) -> axum::response::Response {
//     let id = format!("{id:04x}", id = rand::random::<u16>());
//     tracing::info!(%id, mode = T::mode(), "Opening websocket");
//
//     let broadcast = service.subscribe();
//
//     upgrade.on_upgrade(|socket| handler(service, broadcast, socket::Socket::<T>::new(socket), id))
// }
//
// enum FlowControl<T> {
//     Break,
//     Continue,
//     Pass(T),
// }
//
// #[tracing::instrument(skip_all, fields(%id, mode = T::mode()))]
// async fn handler<T: socket::Payload>(
//     service: Service,
//     mut broadcast: tokio::sync::broadcast::Receiver<Push>,
//     mut socket: socket::Socket<T>,
//     id: String,
// ) {
//     loop {
//         tokio::select! {
//             () = tokio::time::sleep(std::time::Duration::from_secs(30)) => socket.heartbeat().await,
//             message = broadcast.recv() => {
//                 let push = pass!(push(message));
//                 pass!(socket.push(push).await);
//             }
//             request = socket.recv() => {
//                 let request = pass!(request);
//                 let response = service.handle(request).await;
//                 pass!(socket.reply(response).await);
//             }
//         }
//     }
// }
//
// fn push(message: Result<Push, tokio::sync::broadcast::error::RecvError>) -> FlowControl<Push> {
//     match message {
//         Ok(message) => FlowControl::Pass(message),
//         Err(error) => {
//             tracing::warn!(%error, "Failed to read from service listener");
//             FlowControl::Continue
//         }
//     }
// }
