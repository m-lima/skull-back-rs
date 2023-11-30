use crate::{Occurrence, OccurrenceId, Quick, QuickId, Response, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WithId<T> {
    pub id: Option<u32>,
    #[serde(flatten)]
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Push(Push),
    Response(WithId<Response>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Push {
    SkullCreated(Skull),
    SkullUpdated(Skull),
    SkullDeleted(SkullId),
    QuickCreated(Quick),
    QuickUpdated(Quick),
    QuickDeleted(QuickId),
    OccurrencesCreated(Vec<Occurrence>),
    OccurrenceUpdated(Occurrence),
    OccurrenceDeleted(OccurrenceId),
}
