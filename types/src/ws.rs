use crate::{Error, Occurrence, OccurrenceId, Payload, Quick, QuickId, Skull, SkullId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Error(Error),
    Push(Push),
    #[serde(untagged)]
    Payload(Payload),
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
