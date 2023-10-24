use crate::{Id, Occurrence, OccurrenceId, Quick, QuickId, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Response<E> {
    Ok,
    Push(Push),
    Id(Id),
    Skulls(Vec<Skull>),
    Quicks(Vec<Quick>),
    Occurrences(Vec<Occurrence>),
    Error(E),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Payload {
    Ok,
    Push(Push),
    Id(Id),
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
    OccurrenceCreated(Occurrence),
    OccurrenceUpdated(Occurrence),
    OccurrenceDeleted(OccurrenceId),
}

impl<E> From<Result<Payload, E>> for Response<E> {
    fn from(value: Result<Payload, E>) -> Self {
        match value {
            Ok(payload) => match payload {
                Payload::Ok => Self::Ok,
                Payload::Push(push) => Self::Push(push),
                Payload::Id(id) => Self::Id(id),
                Payload::Skulls(skulls) => Self::Skulls(skulls),
                Payload::Quicks(quicks) => Self::Quicks(quicks),
                Payload::Occurrences(occurrences) => Self::Occurrences(occurrences),
            },
            Err(error) => Self::Error(error),
        }
    }
}
