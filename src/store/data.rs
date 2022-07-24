use super::Id;

pub trait Data:
    Clone + Send + Sync + Unpin + PartialEq + std::fmt::Debug + for<'de> serde::Deserialize<'de>
{
    type Id: WithId<Self>;
}

pub trait WithId<D: Data>:
    Clone + Send + Sync + Unpin + PartialEq + PartialEq<D> + std::fmt::Debug + serde::Serialize
{
    fn new(id: Id, data: D) -> Self;
    fn forget_id(self) -> D;
    fn id(&self) -> Id;
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, skull_derive::Data)]
pub struct Skull {
    pub(super) name: String,
    pub(super) color: String,
    pub(super) icon: String,
    #[serde(rename = "unitPrice")]
    pub(super) unit_price: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) limit: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, skull_derive::Data)]
pub struct Quick {
    pub(super) skull: Id,
    pub(super) amount: f32,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, skull_derive::Data)]
pub struct Occurrence {
    pub(super) skull: Id,
    pub(super) amount: f32,
    pub(super) millis: i64,
}

#[cfg(test)]
mod test {
    use super::{Occurrence, OccurrenceId, Quick, QuickId, Skull, SkullId};

    #[test]
    fn serialize_skull() {
        let skull = SkullId {
            id: 3,
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 0.1,
            limit: None,
        };

        assert_eq!(
            serde_json::to_string(&skull).unwrap(),
            r#"{"id":3,"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":0.1}"#
        );
    }

    #[test]
    fn serialize_quick() {
        let quick = QuickId {
            id: 3,
            skull: 1,
            amount: 2.0,
        };

        assert_eq!(
            serde_json::to_string(&quick).unwrap(),
            r#"{"id":3,"skull":1,"amount":2.0}"#
        );
    }

    #[test]
    fn serialize_occurrence() {
        let occurrence = OccurrenceId {
            id: 3,
            skull: 1,
            amount: 2.0,
            millis: 4,
        };

        assert_eq!(
            serde_json::to_string(&occurrence).unwrap(),
            r#"{"id":3,"skull":1,"amount":2.0,"millis":4}"#
        );
    }

    #[test]
    fn deserialize_skull() {
        let json = r#"{"name":"xnamex","color":"xcolorx","icon":"xiconx","unitPrice":1}"#;

        let skull = Skull {
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 1.0,
            limit: None,
        };

        assert_eq!(serde_json::from_str::<Skull>(json).unwrap(), skull);
    }

    #[test]
    fn deserialize_quick() {
        let json = r#"{"skull":1,"amount":2}"#;

        let quick = Quick {
            skull: 1,
            amount: 2.0,
        };

        assert_eq!(serde_json::from_str::<Quick>(json).unwrap(), quick);
    }

    #[test]
    fn deserialize_occurrence() {
        let json = r#"{"skull":1,"amount":2,"millis":4}"#;

        let occurrence = Occurrence {
            skull: 1,
            amount: 2.0,
            millis: 4,
        };

        assert_eq!(
            serde_json::from_str::<Occurrence>(json).unwrap(),
            occurrence
        );
    }
}
