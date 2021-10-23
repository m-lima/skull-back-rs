// Allowed because of proc-macro
#![allow(clippy::trait_duplication_in_bounds)]

use super::Id;

pub trait Data: Clone + serde::Serialize + for<'de> serde::Deserialize<'de> {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct WithId<D: Data> {
    pub(super) id: Id,
    // Also possible with #[serde(deserialize_with = "serde::de::Deserialize::deserialize")]
    // but more code is generated
    #[serde(flatten, bound = "D: Data")]
    pub(super) data: D,
}

impl<D: Data> WithId<D> {
    pub(super) fn new(id: Id, data: D) -> Self {
        Self { id, data }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    pub(super) name: String,
    pub(super) color: String,
    pub(super) icon: String,
    #[serde(rename = "unitPrice")]
    pub(super) unit_price: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) limit: Option<f32>,
}

impl Data for Skull {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    pub(super) skull: Id,
    pub(super) amount: f32,
}

impl Data for Quick {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    pub(super) skull: Id,
    pub(super) amount: f32,
    pub(super) millis: u64,
}

impl Data for Occurrence {}

#[cfg(test)]
mod test {
    use super::{Occurrence, Quick, Skull, WithId};

    #[test]
    fn serialize_skull() {
        let skull = Skull {
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 0.1,
            limit: None,
        };
        let skull_id = WithId::new(3, skull.clone());

        assert_eq!(
            serde_json::to_string(&skull).unwrap(),
            r#"{"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":0.1}"#
        );
        assert_eq!(
            serde_json::to_string(&skull_id).unwrap(),
            r#"{"id":3,"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":0.1}"#
        );
    }

    #[test]
    fn serialize_quick() {
        let quick = Quick {
            skull: 1,
            amount: 2.0,
        };
        let quick_id = WithId::new(3, quick.clone());

        assert_eq!(
            serde_json::to_string(&quick).unwrap(),
            r#"{"skull":1,"amount":2.0}"#
        );
        assert_eq!(
            serde_json::to_string(&quick_id).unwrap(),
            r#"{"id":3,"skull":1,"amount":2.0}"#
        );
    }

    #[test]
    fn serialize_occurrence() {
        let occurrence = Occurrence {
            skull: 1,
            amount: 2.0,
            millis: 4,
        };
        let occurrence_id = WithId::new(3, occurrence.clone());

        assert_eq!(
            serde_json::to_string(&occurrence).unwrap(),
            r#"{"skull":1,"amount":2.0,"millis":4}"#
        );
        assert_eq!(
            serde_json::to_string(&occurrence_id).unwrap(),
            r#"{"id":3,"skull":1,"amount":2.0,"millis":4}"#
        );
    }

    #[test]
    fn deserialize_skull() {
        let json = r#"{"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":1}"#;
        let json_id = r#"{"id":3,"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":1}"#;

        let skull = Skull {
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 1.0,
            limit: None,
        };
        let skull_id = WithId::new(3, skull.clone());

        assert_eq!(serde_json::from_str::<Skull>(json).unwrap(), skull);
        assert_eq!(
            serde_json::from_str::<WithId<Skull>>(json_id).unwrap(),
            skull_id
        );
    }

    #[test]
    fn deserialize_quick() {
        let json = r#"{"skull":1,"amount":2}"#;
        let json_id = r#"{"id":3,"skull":1,"amount":2}"#;

        let quick = Quick {
            skull: 1,
            amount: 2.0,
        };
        let quick_id = WithId::new(3, quick.clone());

        assert_eq!(serde_json::from_str::<Quick>(json).unwrap(), quick);
        assert_eq!(
            serde_json::from_str::<WithId<Quick>>(json_id).unwrap(),
            quick_id
        );
    }

    #[test]
    fn deserialize_occurrence() {
        let json = r#"{"skull":1,"amount":2,"millis":4}"#;
        let json_id = r#"{"id":3,"skull":1,"amount":2,"millis":4}"#;

        let occurrence = Occurrence {
            skull: 1,
            amount: 2.0,
            millis: 4,
        };
        let occurrence_id = WithId::new(3, occurrence.clone());

        assert_eq!(
            serde_json::from_str::<Occurrence>(json).unwrap(),
            occurrence
        );
        assert_eq!(
            serde_json::from_str::<WithId<Occurrence>>(json_id).unwrap(),
            occurrence_id
        );
    }
}
