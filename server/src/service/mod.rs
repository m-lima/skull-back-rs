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

    async fn send(&self, push: types::Push) {
        if let Ok(count) = self.sender.send(push) {
            if count == 1 {
                tracing::debug!("Broadcasting to 1 listener");
            } else {
                tracing::debug!("Broadcasting to {count} listeners");
            }
        }
    }
}

type Result = std::result::Result<types::Payload, store::Error>;

impl Service {
    pub async fn handle(&self, request: types::Request) -> Result {
        match request {
            types::Request::Skull(request) => skulls::handle(self, request).await,
            types::Request::Quick(request) => quicks::handle(self, request).await,
            types::Request::Occurrence(request) => occurrences::handle(self, request).await,
        }
    }

    async fn new<P: AsRef<std::path::Path>>(path: P) -> store::Result<Self> {
        let store = store::Store::new(path, 1).await?;
        store.migrate().await?;

        let broadcaster = Broadcaster::new();

        Ok(Self { store, broadcaster })
    }
}
