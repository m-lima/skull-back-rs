mod flow;
mod mode;

pub use mode::Mode;

use crate::service::Service;

enum FlowControl<T> {
    Break,
    Continue,
    Pass(T),
}

pub struct Socket<T: Mode> {
    id: String,
    socket: axum::extract::ws::WebSocket,
    service: Service,
    _mode: std::marker::PhantomData<T>,
}

impl<T: Mode> Socket<T> {
    pub fn new(socket: axum::extract::ws::WebSocket, service: Service) -> Self {
        let id = format!("{id:04x}", id = rand::random::<u16>());
        tracing::debug!(ws = %id, mode = %T::mode(), "Opening websocket");

        Self {
            id,
            socket,
            service,
            _mode: std::marker::PhantomData,
        }
    }

    #[tracing::instrument(target = "ws", skip_all, fields(ws = %self.id, mode = %T::mode()))]
    pub async fn serve(mut self) {
        macro_rules! flow {
            ($flow_control: expr) => {
                match $flow_control {
                    FlowControl::Pass(value) => value,
                    FlowControl::Break => break,
                    FlowControl::Continue => continue,
                }
            };
        }

        let mut broadcast = self.service.subscribe();

        loop {
            tokio::select! {
                () = tokio::time::sleep(std::time::Duration::from_secs(30)) => self.heartbeat().await,
                message = broadcast.recv() => {
                    let push = match message {
                        Ok(push) => types::Message::Push(push),
                        Err(error) => {
                            tracing::warn!(ws = %self.id, mode = %T::mode(), %error, "Failed to read from broadcaster");
                            continue;
                        }
                    };
                    tracing::debug!(ws = %self.id, mode = %T::mode(), "Pushing message");
                    flow!(self.send(push).await);
                }
                request = self.recv() => {
                    let start = std::time::Instant::now();
                    let request = flow!(request);
                    let (resource, action) = flow::incoming(&request);
                    let response = self.service.handle(request).await;
                    let outgoing = flow::outgoing(&response);

                    let message = match response {
                        types::Response::Payload(payload) => {
                            tracing::info!(ws = %self.id, mode = %T::mode(), %resource, %action, latency = ?start.elapsed(), "{outgoing}");
                            types::Message::Payload(payload)
                        }
                        types::Response::Error(error) => {
                            if error.kind == types::Kind::InternalError {
                                tracing::error!(ws = %self.id, mode = %T::mode(), %resource, %action, latency = ?start.elapsed(), "{outgoing}");
                            } else {
                                tracing::warn!(ws = %self.id, mode = %T::mode(), %resource, %action, latency = ?start.elapsed(), "{outgoing}");
                            }
                            types::Message::Error(error)
                        }
                    };
                    flow!(self.send(message).await);
                }
            }
        }
    }

    async fn heartbeat(&mut self) {
        tracing::debug!("Sending heartbeat");
        if let Err(error) = self
            .socket
            .send(axum::extract::ws::Message::Ping(Vec::new()))
            .await
        {
            tracing::warn!(%error, "Failed to send heartbeat");
        }
    }

    async fn recv(&mut self) -> FlowControl<types::Request> {
        // Closed socket
        let Some(message) = self.socket.recv().await else {
            tracing::debug!(ws = %self.id, mode = %T::mode(), "Closing websocket");
            return FlowControl::Break;
        };

        // Broken socket
        let message = match message {
            Ok(message) => message,
            Err(error) => {
                tracing::warn!(ws = %self.id, mode = %T::mode(), %error, "Broken websocket");
                return FlowControl::Break;
            }
        };

        let bytes = match message {
            // Control messages
            axum::extract::ws::Message::Ping(_) | axum::extract::ws::Message::Pong(_) => {
                tracing::debug!(ws = %self.id, mode = %T::mode(), "Received ping");
                return FlowControl::Continue;
            }
            axum::extract::ws::Message::Close(_) => {
                tracing::debug!(ws = %self.id, mode = %T::mode(), "Received close request");
                return FlowControl::Continue;
            }

            // Payload messages
            axum::extract::ws::Message::Text(text) => text.into_bytes(),
            axum::extract::ws::Message::Binary(binary) => binary,
        };

        match T::deserialize(bytes) {
            Ok(request) => FlowControl::Pass(request),
            Err(error) => {
                tracing::warn!(ws = %self.id, mode = %T::mode(), %error, "Failed to deserialize request");
                let error = types::Error {
                    kind: types::Kind::BadRequest,
                    message: Some(error.to_string()),
                };
                self.send(types::Message::Error(error)).await
            }
        }
    }

    async fn send<R>(&mut self, response: types::Message) -> FlowControl<R> {
        match T::serialize(response) {
            Ok(response) => {
                if let Err(error) = self.socket.send(response).await {
                    tracing::error!(ws = %self.id, mode = %T::mode(), %error, "Failed to send message");
                    FlowControl::Break
                } else {
                    FlowControl::Continue
                }
            }
            Err(error) => {
                tracing::error!(ws = %self.id, mode = %T::mode(), %error, "Failed to serialize message");
                FlowControl::Break
            }
        }
    }
}
