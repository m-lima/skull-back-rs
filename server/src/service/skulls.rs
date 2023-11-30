use types::{
    request::{
        skull::{Create, Delete, Update},
        Skull,
    },
    Change, Payload, Push, Setter,
};

use super::{Broadcaster, Service};

type Result = std::result::Result<types::Payload, store::Error>;

pub async fn handle(service: &Service, request: Skull) -> Result {
    let skulls = Skulls::new(service);
    match request {
        Skull::List => skulls.list().await,
        Skull::Create(request) => skulls.create(request).await,
        Skull::Update(request) => skulls.update(request).await,
        Skull::Delete(request) => skulls.delete(request).await,
    }
}

struct Skulls<'a> {
    store: store::store::skulls::Skulls<'a>,
    broadcaster: &'a Broadcaster,
}

impl<'a> Skulls<'a> {
    fn new(service: &'a Service) -> Self {
        let store = service.store.skulls();
        let broadcaster = &service.broadcaster;
        Self { store, broadcaster }
    }
}

impl Skulls<'_> {
    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Skulls)
    }

    async fn create(&self, request: Create) -> Result {
        let created = self
            .store
            .create(
                request.name,
                request.color,
                request.icon,
                request.price,
                request.limit,
            )
            .await?;

        self.broadcaster.send(Push::SkullCreated(created));
        Ok(Payload::Change(Change::Created))
    }

    async fn update(&self, request: Update) -> Result {
        let updated = self
            .store
            .update(
                request.id,
                request.name.map(Setter::set),
                request.color.map(Setter::set),
                request.icon.map(Setter::set),
                request.price.map(Setter::set),
                request.limit.map(Setter::set),
            )
            .await?;

        self.broadcaster.send(Push::SkullUpdated(updated));
        Ok(Payload::Change(Change::Updated))
    }

    async fn delete(&self, request: Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster.send(Push::SkullDeleted(request.id));
        Ok(Payload::Change(Change::Deleted))
    }
}
