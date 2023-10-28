use types::{
    request::{Quick, Setter},
    Payload, Push,
};

use super::{Action, Broadcaster, Resource, Response, Result, Service};

pub struct Quicks<'a> {
    store: store::store::quicks::Quicks<'a>,
    broadcaster: &'a Broadcaster,
}

impl<'a> Quicks<'a> {
    pub fn new(service: &'a Service) -> Self {
        let store = service.store.quicks();
        let broadcaster = &service.broadcaster;
        Self { store, broadcaster }
    }
}

impl Quicks<'_> {
    pub async fn handle(&self, request: Quick) -> Response {
        let (action, result) = match request {
            Quick::List => (Action::List, self.list().await),
            Quick::Create(request) => (Action::Create, self.create(request).await),
            Quick::Update(request) => (Action::Update, self.update(request).await),
            Quick::Delete(request) => (Action::Delete, self.delete(request).await),
        };

        Response {
            resource: Resource::Quick,
            action,
            result,
        }
    }

    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Quicks)
    }

    async fn create(&self, request: types::request::quick::Create) -> Result {
        let created = self.store.create(request.skull, request.amount).await?;

        let id = created.id;
        self.broadcaster.send(Push::QuickCreated(created)).await;
        Ok(Payload::Quick(id))
    }

    async fn update(&self, request: types::request::quick::Update) -> Result {
        let updated = self
            .store
            .update(
                request.id,
                request.skull.map(Setter::set),
                request.amount.map(Setter::set),
            )
            .await?;

        self.broadcaster.send(Push::QuickUpdated(updated)).await;
        Ok(Payload::Ok)
    }

    async fn delete(&self, request: types::request::quick::Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster.send(Push::QuickDeleted(request.id)).await;
        Ok(Payload::Ok)
    }
}
