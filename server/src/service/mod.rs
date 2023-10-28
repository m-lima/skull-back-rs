mod skull;
mod quick;
mod occurrence;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Resource {
    Skull,
    Quick,
    Occurrence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Action {
    List,
    Search,
    Create,
    Update,
    Delete,
}

type Result = std::result::Result<types::Payload, store::Error>;

struct Response {
    resource: Resource,
    action: Action,
    result: Result,
}

impl Service {
    pub async fn handle(&self, request: types::Request) {
        let result = match request {
            types::Request::Skull(request) => skull::Skulls::new(self).handle(request).await,
            types::Request::Quick(request) => todo!()
            types::Request::Occurrence(request) => todo!()
        };

        match result.2 {
            Ok(payload) => todo!(),
            Err(_) => todo!(),
        }
    }
}

impl Service {
    async fn new<P: AsRef<std::path::Path>>(path: P) -> store::Result<Self> {
        let store = store::Store::new(path, 1).await?;
        store.migrate().await?;

        let broadcaster = Broadcaster::new();

        Ok(Self { store, broadcaster })
    }

    async fn skull(&self, request: types::request::Skull) -> Response {
        use types::{
            request::{Setter, Skull},
            response::Payload,
        };

        let (action, response) = match request {
            Skull::List => (
                Action::List,
                self.store.skulls().list().await.map(Payload::Skulls),
            ),
            Skull::Create(create) => (
                Action::Create,
                self.store
                    .skulls()
                    .create(
                        create.name,
                        create.color,
                        create.icon,
                        create.unit_price,
                        create.limit,
                    )
                    .await
                    .map(|created| {
                        let id = created.id;
                        self.broadcast(types::Push::SkullCreated(created));
                        Payload::Skull(id)
                    }),
            ),
            Skull::Update(update) => (
                Action::Update,
                self.store
                    .skulls()
                    .update(
                        update.id,
                        update.name.map(Setter::set),
                        update.color.map(Setter::set),
                        update.icon.map(Setter::set),
                        update.unit_price.map(Setter::set),
                        update.limit.map(Setter::set),
                    )
                    .await
                    .map(|updated| {
                        self.broadcast(types::Push::SkullUpdated(updated));
                        Payload::Ok
                    }),
            ),
            Skull::Delete(delete) => (
                Action::Delete,
                self.store.skulls().delete(delete.id).await.map(|_| {
                    self.broadcast(types::Push::SkullDeleted(delete.id));
                    Payload::Ok
                }),
            ),
        };

        Response {
            resource: Resource::Skull,
            action,
            response,
        }
    }

    async fn quick(&self, request: types::request::Quick) -> Response {
        use types::{
            request::{Quick, Setter},
            response::Payload,
        };

        let (action, response) = match request {
            Quick::List => (
                Action::List,
                self.store.quicks().list().await.map(Payload::Quicks),
            ),
            Quick::Create(create) => (
                Action::Create,
                self.store
                    .quicks()
                    .create(create.skull, create.amount)
                    .await
                    .map(|created| {
                        let id = created.id;
                        self.broadcast(types::Push::QuickCreated(created));
                        Payload::Quick(id)
                    }),
            ),
            Quick::Update(update) => (
                Action::Update,
                self.store
                    .quicks()
                    .update(
                        update.id,
                        update.skull.map(Setter::set),
                        update.amount.map(Setter::set),
                    )
                    .await
                    .map(|updated| {
                        self.broadcast(types::Push::QuickUpdated(updated));
                        Payload::Ok
                    }),
            ),
            Quick::Delete(delete) => (
                Action::Delete,
                self.store.quicks().delete(delete.id).await.map(|_| {
                    self.broadcast(types::Push::QuickDeleted(delete.id));
                    Payload::Ok
                }),
            ),
        };

        Response {
            resource: Resource::Quick,
            action,
            response,
        }
    }

    async fn occurence(&self, request: types::request::Occurrence) -> Response {
        use types::{
            request::{Occurrence, Setter},
            response::Payload,
        };

        let (action, response) = match request {
            Occurrence::List => (
                Action::List,
                self.store
                    .occurrences()
                    .list()
                    .await
                    .map(Payload::Occurrences),
            ),
            Occurrence::Search(search) => (
                Action::Search,
                self.store
                    .occurrences()
                    .search(
                        search.skulls.as_ref(),
                        search.start,
                        search.end,
                        search.limit,
                    )
                    .await
                    .map(Payload::Occurrences),
            ),
            Occurrence::Create(create) => (
                Action::Create,
                self.store
                    .occurrences()
                    .create(
                        create
                            .items
                            .into_iter()
                            .map(|item| (item.skull, item.amount, item.millis)),
                    )
                    .await
                    .map(|created| {
                        let ids = created.iter().map(|occurrence| occurrence.id).collect();
                        self.broadcast(types::Push::OccurrencesCreated(created));
                        Payload::Ids(ids)
                    }),
            ),
            Occurrence::Update(update) => (
                Action::Update,
                self.store
                    .occurrences()
                    .update(
                        update.id,
                        update.skull.map(Setter::set),
                        update.amount.map(Setter::set),
                        update.millis.map(Setter::set),
                    )
                    .await
                    .map(|updated| {
                        self.broadcast(types::Push::OccurrenceUpdated(updated));
                        Payload::Ok
                    }),
            ),
            Occurrence::Delete(delete) => (
                Action::Delete,
                self.store.occurrences().delete(delete.id).await.map(|_| {
                    self.broadcast(types::Push::OccurrenceDeleted(delete.id));
                    Payload::Ok
                }),
            ),
        };

        Response {
            resource: Resource::Occurrence,
            action,
            response,
        }
    }
}

pub async fn create_all(
    users: std::collections::HashSet<String>,
    db_root: std::path::PathBuf,
) -> store::Result<std::collections::HashMap<String, Service>> {
    let mut services = std::collections::HashMap::with_capacity(users.len());

    for user in users {
        let service = Service::new(db_root.join(&user)).await?;
        services.insert(user, service);
    }

    Ok(services)
}

pub fn prepare_users(
    create: bool,
    db_root: &std::path::Path,
    users: &std::collections::HashSet<String>,
) -> bool {
    if users.is_empty() {
        tracing::error!("No users provided");
        return false;
    }

    if create {
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
    }

    true
}

// impl tower_service::Service<types::Request> for Service {
//     type Response = Response;
//     type Error = std::convert::Infallible;
//     type Future = std::pin::Pin<Box<dyn std::future::Future<Output = InfallibleResponse>>>;
//
//     fn poll_ready(
//         &mut self,
//         _: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, request: types::Request) -> Self::Future {
//         match request {
//             types::Request::Skull(request) => Box::pin(self.skull(request)),
//             types::Request::Quick(request) => Box::pin(self.quick(request)),
//             types::Request::Occurrence(request) => Box::pin(self.occurence(request)),
//         }
//     }
// }
