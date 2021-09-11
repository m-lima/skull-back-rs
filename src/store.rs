mod in_memory;

pub type Id = u32;

macro_rules! impl_store_data {
    ($name:ty) => {
        impl Data for $name {}
    };
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Entry not found for id `{0}`")]
    NotFound(Id),
    #[error("Store full")]
    StoreFull,
}

pub trait Data {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    name: String,
    price: f32,
}

impl_store_data!(Skull);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    skull: Skull,
    amount: f32,
}

impl_store_data!(Quick);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    skull: Skull,
    amount: f32,
    time: std::time::SystemTime,
}

impl_store_data!(Occurrence);

pub trait Store: Send + 'static {
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

pub trait Crud<D: Data> {
    fn list(&self) -> Result<Vec<(&Id, &D)>, Error>;
    fn filter_list(&self, filter: Box<dyn Fn(&D) -> bool>) -> Result<Vec<(&Id, &D)>, Error>;
    fn create(&mut self, data: D) -> Result<Id, Error>;
    fn read(&self, id: Id) -> Result<&D, Error>;
    fn update(&mut self, id: Id, data: D) -> Result<D, Error>;
    fn delete(&mut self, id: Id) -> Result<D, Error>;
}

pub fn in_memory() -> in_memory::InMemory {
    in_memory::InMemory::default()
}
