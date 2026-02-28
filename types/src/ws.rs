use crate::{Occurrence, OccurrenceId, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WithId<T> {
    pub id: Option<u32>,
    #[serde(flatten)]
    pub payload: T,
}

pub type Request = WithId<crate::Request>;
pub type Response = WithId<crate::Response>;
pub type Id = WithId<()>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Push(Push),
    Response(Response),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Push {
    SkullCreated(Skull),
    SkullUpdated(Skull),
    SkullDeleted(SkullId),
    OccurrencesCreated(Vec<Occurrence>),
    OccurrenceUpdated(Occurrence),
    OccurrenceDeleted(OccurrenceId),
}
