mod in_file;
mod in_memory;

mod id_setter {
    pub trait IdSetter {
        fn set_id(&mut self, id: super::Id);
    }
}

use id_setter::IdSetter;

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

pub trait Data:
    id_setter::IdSetter + Clone + serde::Serialize + serde::de::DeserializeOwned
{
    fn id(&self) -> Id;
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    #[serde(default)]
    id: Id,
    name: String,
    color: String,
    icon: String,
    #[serde(rename = "unitPrice")]
    unit_price: f32,
    limit: Option<f32>,
}

impl Data for Skull {
    fn id(&self) -> Id {
        self.id
    }
}

impl IdSetter for Skull {
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    #[serde(default)]
    id: Id,
    skull: Id,
    amount: f32,
}

impl Data for Quick {
    fn id(&self) -> Id {
        self.id
    }
}

impl IdSetter for Quick {
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    #[serde(default)]
    id: Id,
    skull: Id,
    amount: f32,
    #[serde(rename = "millis", with = "time")]
    timestamp: std::time::SystemTime,
}

impl Data for Occurrence {
    fn id(&self) -> Id {
        self.id
    }
}

impl IdSetter for Occurrence {
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

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
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, D>>, Error>;
    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&D) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, D>>, Error>;
    fn create(&mut self, user: &str, data: D) -> Result<Id, Error>;
    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, D>, Error>;
    fn update(&mut self, user: &str, id: Id, data: D) -> Result<D, Error>;
    fn delete(&mut self, user: &str, id: Id) -> Result<D, Error>;
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
