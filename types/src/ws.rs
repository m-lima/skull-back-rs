use crate::{Occurrence, OccurrenceId, Quick, QuickId, Request, Response, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RequestId {
    pub id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RequestWithId {
    pub id: Option<u32>,
    #[serde(flatten)]
    pub payload: Request,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResponseWithId {
    pub id: Option<u32>,
    #[serde(flatten)]
    pub payload: Response,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Push(Push),
    Response(ResponseWithId),
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
