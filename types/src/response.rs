use crate::{Error, Occurrence, Quick, Skull};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Error(Error),
    #[serde(untagged)]
    Payload(Payload),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Payload {
    Change(Change),
    Skulls(Vec<Skull>),
    Quicks(Vec<Quick>),
    Occurrences(Vec<Occurrence>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Change {
    Created,
    Updated,
    Deleted,
}
