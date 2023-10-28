use types::{
    request::{Occurrence, Setter},
    Payload, Push,
};

use super::{Action, Broadcaster, Resource, Response, Result, Service};

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
    pub async fn handle(&self, request: Occurrence) -> Response {
        let (action, result) = match request {
            Occurrence::List => (Action::List, self.list().await),
            Occurrence::Search(request) => (Action::Search, self.search(request).await),
            Occurrence::Create(request) => (Action::Create, self.create(request).await),
            Occurrence::Update(request) => (Action::Update, self.update(request).await),
            Occurrence::Delete(request) => (Action::Delete, self.delete(request).await),
        };

        Response {
            resource: Resource::Occurrence,
            action,
            result,
        }
    }

    async fn list(&self) -> Result {
        self.store.list().await.map(Payload::Occurrences)
    }

    async fn search(&self, request: types::request::occurrence::Search) -> Result {
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

    async fn create(&self, request: types::request::occurrence::Create) -> Result {
        let created = self
            .store
            .create(
                request
                    .items
                    .into_iter()
                    .map(|item| (item.skull, item.amount, item.millis)),
            )
            .await?;

        let ids = created.iter().map(|occurrence| occurrence.id).collect();
        self.broadcaster.send(Push::OccurrencesCreated(created));
        Ok(Payload::Ids(ids))
    }

    async fn update(&self, request: types::request::occurrence::Update) -> Result {
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
        Ok(Payload::Ok)
    }

    async fn delete(&self, request: types::request::occurrence::Delete) -> Result {
        self.store.delete(request.id).await?;

        self.broadcaster
            .send(Push::OccurrenceDeleted(request.id))
            .await;
        Ok(Payload::Ok)
    }
}
