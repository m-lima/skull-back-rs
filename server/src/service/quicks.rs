use types::{
    request::{
        quick::{Create, Delete, Update},
        Quick, Setter,
    },
    Payload, Push,
};

use super::{Broadcaster, Result, Service};

pub async fn handle(service: &Service, request: Quick) -> Result {
    let quicks = Quicks::new(service);
    match request {
        Quick::List => quicks.list().await,
        Quick::Create(request) => quicks.create(request).await,
        Quick::Update(request) => quicks.update(request).await,
        Quick::Delete(request) => quicks.delete(request).await,
    }
}

pub struct Quicks<'a> {
    store: store::store::quicks::Quicks<'a>,
    broadcaster: &'a Broadcaster,
}

impl<'a> Quicks<'a> {
    fn new(service: &'a Service) -> Self {
        let store = service.store.quicks();
        let broadcaster = &service.broadcaster;
        Self { store, broadcaster }
    }
}

impl Quicks<'_> {
    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Quicks)
    }

    async fn create(&self, request: Create) -> Result {
        let created = self.store.create(request.skull, request.amount).await?;

        self.broadcaster.send(Push::QuickCreated(created)).await;
        Ok(Payload::Created)
    }

    async fn update(&self, request: Update) -> Result {
        let updated = self
            .store
            .update(
                request.id,
                request.skull.map(Setter::set),
                request.amount.map(Setter::set),
            )
            .await?;

        self.broadcaster.send(Push::QuickUpdated(updated)).await;
        Ok(Payload::Updated)
    }

    async fn delete(&self, request: Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster.send(Push::QuickDeleted(request.id)).await;
        Ok(Payload::Deleted)
    }
}
