// Allowed because of proc-macro
#![allow(clippy::trait_duplication_in_bounds)]

mod in_file;
mod in_memory;

pub type Id = u32;

pub fn in_memory<S, I>(users: I) -> impl Store
where
    S: ToString,
    I: std::iter::IntoIterator<Item = S>,
{
    in_memory::InMemory::new(users)
}

pub fn in_file<S, I, P>(path: P, users: I) -> anyhow::Result<impl Store>
where
    S: AsRef<str>,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    in_file::InFile::new(path, users)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User not found `{0}`")]
    NoSuchUser(String),
    #[error("Entry not found for id `{0}`")]
    NotFound(Id),
    #[error("Store full")]
    StoreFull,
    #[error("{0}")]
    Io(std::io::Error),
    #[error("Serde error: {0}")]
    Serde(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub trait Data: Clone + serde::Serialize + for<'de> serde::Deserialize<'de> {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct WithId<D: Data> {
    id: Id,
    // Also possible with #[serde(deserialize_with = "serde::de::Deserialize::deserialize")]
    // but more code is generated
    #[serde(flatten, bound = "D: Data")]
    data: D,
}

impl<D: Data> WithId<D> {
    fn new(id: Id, data: D) -> Self {
        Self { id, data }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    name: String,
    color: String,
    icon: String,
    #[serde(rename = "unitPrice")]
    unit_price: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<f32>,
}

impl Data for Skull {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    skull: Id,
    amount: f32,
}

impl Data for Quick {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    skull: Id,
    amount: f32,
    #[serde(rename = "millis", with = "time")]
    timestamp: std::time::SystemTime,
}

impl Data for Occurrence {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct LastModified {
    #[serde(rename = "millis", with = "time")]
    timestamp: std::time::SystemTime,
}

pub trait Store: Send + 'static {
    fn last_modified(&self, user: &str) -> Result<LastModified, Error>;
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

mod time {
    pub fn serialize<S>(time: &std::time::SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let millis = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("Time is before UNIX_EPOCH"))?
            .as_millis();

        serializer.serialize_u128(millis)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<std::time::SystemTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let millis = <u64 as serde::Deserialize>::deserialize(deserializer)?;
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_millis(millis))
            .ok_or_else(|| serde::de::Error::custom("Could not parse UNIX EPOCH"))
    }
}

// TODO: When using a RDB, will this interface still make sense?
// TODO: Is it possible to avoid the Vec's?
pub trait Crud<D: Data> {
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error>;
    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&WithId<D>) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error>;
    fn create(&mut self, user: &str, data: D) -> Result<Id, Error>;
    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, WithId<D>>, Error>;
    fn update(&mut self, user: &str, id: Id, data: D) -> Result<WithId<D>, Error>;
    fn delete(&mut self, user: &str, id: Id) -> Result<WithId<D>, Error>;
}

pub trait CrudSelector: Data {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self>;
}

impl CrudSelector for Skull {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.skull()
    }
}

impl CrudSelector for Quick {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.quick()
    }
}

impl CrudSelector for Occurrence {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.occurrence()
    }
}

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
            timestamp: std::time::UNIX_EPOCH
                .checked_add(std::time::Duration::from_millis(4))
                .unwrap(),
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
            timestamp: std::time::UNIX_EPOCH
                .checked_add(std::time::Duration::from_millis(4))
                .unwrap(),
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
