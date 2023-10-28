mod occurrences;
mod quicks;
mod skulls;

pub async fn new(
    db_root: std::path::PathBuf,
    users: std::collections::HashSet<String>,
) -> store::Result<std::collections::HashMap<String, Service>> {
    let mut services = std::collections::HashMap::with_capacity(users.len());

    for user in users {
        let service = Service::new(db_root.join(&user)).await?;
        services.insert(user, service);
    }

    Ok(services)
}

pub fn create_users(db_root: &std::path::Path, users: &std::collections::HashSet<String>) -> bool {
    for user in users {
        let path = db_root.join(user);
        if !path.exists() {
            tracing::info!(db = %path.display(), "Creating database");
            if let Err(error) = std::fs::write(&path, []) {
                tracing::error!(db = %path.display(), %error, "Unable to create database");
                return false;
            }
        }
    }

    true
}

#[derive(Debug, Clone)]
pub struct Service {
    store: store::Store,
    broadcaster: Broadcaster,
}

#[derive(Debug, Clone)]
struct Broadcaster {
    sender: tokio::sync::broadcast::Sender<types::Push>,
}

impl Broadcaster {
    fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel::<types::Push>(16);
        Self { sender }
    }

    fn send(&self, push: types::Push) {
        if let Ok(count) = self.sender.send(push) {
            if count == 1 {
                tracing::debug!("Broadcasting to 1 listener");
            } else {
                tracing::debug!("Broadcasting to {count} listeners");
            }
        }
    }
}

impl Service {
    pub async fn handle(&self, request: types::Request) -> types::Response {
        let result = match request {
            types::Request::Skull(request) => skulls::handle(self, request).await,
            types::Request::Quick(request) => quicks::handle(self, request).await,
            types::Request::Occurrence(request) => occurrences::handle(self, request).await,
        };

        match result {
            Ok(payload) => types::Response::Payload(payload),
            Err(error) => {
                if error.kind() == types::Kind::InternalError {
                    tracing::error!(%error, "Internal error");
                }
                types::Response::Error(error.into())
            }
        }
        // TODO: Log here the type of request
        // TODO: Log here the error (especially if 500)
        // TODO: Suppress top-level logging for REST?
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<types::Push> {
        self.broadcaster.sender.subscribe()
    }

    async fn new<P: AsRef<std::path::Path>>(path: P) -> store::Result<Self> {
        let store = store::Store::new(path, 1).await?;
        store.migrate().await?;

        let broadcaster = Broadcaster::new();

        Ok(Self { store, broadcaster })
    }
}
