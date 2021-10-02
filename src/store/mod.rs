// mod in_file;
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

// pub fn in_file<P: AsRef<std::path::Path>>(path: P) -> in_file::InFile {
//     in_file::InFile::new(path)
// }

// TODO: Should this be a String and let the front end parse it?
pub type Color = u32;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User not found `{0}`")]
    NoSuchUser(String),
    #[error("Entry not found for id `{0}`")]
    NotFound(Id),
    #[error("Store full")]
    StoreFull,
}

pub trait Data: id_setter::IdSetter + serde::Serialize + serde::de::DeserializeOwned {
    fn id(&self) -> Id;
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    id: Id,
    name: String,
    color: Color,
    icon: String,
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
    id: Id,
    skull: Id,
    amount: f32,
    #[serde(with = "time")]
    secs: std::time::SystemTime,
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

pub trait Store: Send + 'static {
    fn last_modified(&self, user: &str) -> Result<std::time::SystemTime, Error>;
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

mod time {
    pub fn serialize<S>(time: &std::time::SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let secs = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("Time is before UNIX_EPOCH"))?
            .as_secs();

        serializer.serialize_u64(secs)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<std::time::SystemTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let secs = <u64 as serde::Deserialize>::deserialize(deserializer)?;
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(secs))
            .ok_or_else(|| serde::de::Error::custom("Could not parse UNIX EPOCH"))
    }
}

// TODO: When using a RDB, will this interface still make sense?
// TODO: Is it possible to avoid the Vec's?
pub trait Crud<D: Data> {
    fn list(&self, user: &str) -> Result<Vec<&D>, Error>;
    fn filter_list(&self, user: &str, filter: Box<dyn Fn(&D) -> bool>) -> Result<Vec<&D>, Error>;
    fn create(&mut self, user: &str, data: D) -> Result<Id, Error>;
    fn read(&self, user: &str, id: Id) -> Result<&D, Error>;
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
