use types::{
    request::{Setter, Skull},
    Payload, Push,
};

use super::{Action, Broadcaster, Resource, Response, Result, Service};

pub struct Skulls<'a> {
    store: store::store::skulls::Skulls<'a>,
    broadcaster: &'a Broadcaster,
}

impl<'a> Skulls<'a> {
    pub fn new(service: &'a Service) -> Self {
        let store = service.store.skulls();
        let broadcaster = &service.broadcaster;
        Self { store, broadcaster }
    }
}

impl Skulls<'_> {
    pub async fn handle(&self, request: Skull) -> Response {
        let (action, result) = match request {
            Skull::List => (Action::List, self.list().await),
            Skull::Create(request) => (Action::Create, self.create(request).await),
            Skull::Update(request) => (Action::Update, self.update(request).await),
            Skull::Delete(request) => (Action::Delete, self.delete(request).await),
        };

        Response {
            resource: Resource::Skull,
            action,
            result,
        }
    }

    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Skulls)
    }

    async fn create(&self, request: types::request::skull::Create) -> Result {
        let created = self
            .store
            .create(
                request.name,
                request.color,
                request.icon,
                request.unit_price,
                request.limit,
            )
            .await?;

        let id = created.id;
        self.broadcaster.send(Push::SkullCreated(created)).await;
        Ok(Payload::Skull(id))
    }

    async fn update(&self, request: types::request::skull::Update) -> Result {
        let updated = self
            .store
            .update(
                request.id,
                request.name.map(Setter::set),
                request.color.map(Setter::set),
                request.icon.map(Setter::set),
                request.unit_price.map(Setter::set),
                request.limit.map(Setter::set),
            )
            .await?;

        self.broadcaster.send(Push::SkullUpdated(updated)).await;
        Ok(Payload::Ok)
    }

    async fn delete(&self, request: types::request::skull::Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster.send(Push::SkullDeleted(request.id)).await;
        Ok(Payload::Ok)
    }
}
