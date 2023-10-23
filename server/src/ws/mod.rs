mod upgrade;

pub use upgrade::Upgrade;

use crate::auth::Session;
use crate::service::Service;

pub struct WebSocket<S>(std::marker::PhantomData<S>);

#[derive(Debug, Copy, Clone)]
enum Mode {
    Text,
    Binary,
}

async fn listen(socket: (), session: Session<Service>, mode: Mode) {
    todo!()
}
