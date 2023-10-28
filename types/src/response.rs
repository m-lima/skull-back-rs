use crate::{Occurrence, OccurrenceId, Quick, QuickId, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Error(Error),
    #[serde(untagged)]
    Ok(Payload),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Payload {
    Created,
    Updated,
    Deleted,
    Skulls(Vec<Skull>),
    Quicks(Vec<Quick>),
    Occurrences(Vec<Occurrence>),
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Error {
    pub kind: Kind,
    pub message: Option<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    BadRequest,
    NotFound,
    InternalError,
}
