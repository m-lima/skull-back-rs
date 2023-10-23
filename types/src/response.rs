use crate::{Id, Occurrence, OccurrenceId, Quick, QuickId, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Response {
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
