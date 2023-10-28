use types::{
    request::{
        occurrence::{Create, Search, Update},
        Occurrence, Setter,
    },
    Payload, Push,
};

use super::{Broadcaster, Result, Service};

pub async fn handle(service: &Service, request: Occurrence) -> Result {
    let occurrences = Occurrences::new(service);
    match request {
        Occurrence::List => occurrences.list().await,
        Occurrence::Search(request) => occurrences.search(request).await,
        Occurrence::Create(request) => occurrences.create(request).await,
        Occurrence::Update(request) => occurrences.update(request).await,
        Occurrence::Delete(request) => occurrences.delete(request).await,
    }
}

pub struct Occurrences<'a> {
    store: store::store::occurrences::Occurrences<'a>,
    broadcaster: &'a Broadcaster,
}

impl<'a> Occurrences<'a> {
    pub fn new(service: &'a Service) -> Self {
        let store = service.store.occurrences();
        let broadcaster = &service.broadcaster;
        Self { store, broadcaster }
    }
}

impl Occurrences<'_> {
    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Occurrences)
    }

    async fn search(&self, request: Search) -> Result {
        self.store
            .search(
                request.skulls.as_ref(),
                request.start,
                request.end,
                request.limit,
            )
            .await
            .map(Payload::Occurrences)
    }

    async fn create(&self, request: Create) -> Result {
        let created = self
            .store
            .create(
                request
                    .items
                    .into_iter()
                    .map(|item| (item.skull, item.amount, item.millis)),
            )
            .await?;

        self.broadcaster.send(Push::OccurrencesCreated(created));
        Ok(Payload::Created)
    }

    async fn update(&self, request: Update) -> Result {
        let updated = self
            .store
            .update(
                request.id,
                request.skull.map(Setter::set),
                request.amount.map(Setter::set),
                request.millis.map(Setter::set),
            )
            .await?;

        self.broadcaster
            .send(Push::OccurrenceUpdated(updated))
            .await;
        Ok(Payload::Updated)
    }

    async fn delete(&self, request: types::request::occurrence::Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster
            .send(Push::OccurrenceDeleted(request.id))
            .await;
        Ok(Payload::Deleted)
    }
}
