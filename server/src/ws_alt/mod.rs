mod upgrade;

pub use upgrade::Upgrade;

use crate::auth::Session;
use crate::service::Service;

pub struct WebSocket<S>(std::marker::PhantomData<S>);

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Text,
    Binary,
}

pub struct Listener;

impl upgrade::Listener<Service> for Listener {
    type Future = ();

    fn listen(
        socket: tokio_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
        session: Session<Service>,
        mode: self::Mode,
    ) -> Self::Future {
        todo!()
    }
}
