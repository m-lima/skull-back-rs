mod in_memory;

pub type Id = u32;

macro_rules! impl_store_data {
    ($name:ty) => {
        impl Data for $name {
            fn id(&self) -> Id {
                self.id
            }
        }

        impl InnerData for $name {
            fn set_id(&mut self, id: Id) {
                self.id = id;
            }
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry not found for id `{0}`")]
    NotFound(Id),
    #[error("store full")]
    StoreFull,
    #[error("unknown error: {0}")]
    Unknown(Box<dyn std::error::Error>),
}

pub trait Data {
    fn id(&self) -> Id;
}

trait InnerData: Data {
    fn set_id(&mut self, id: Id);
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Skull {
    id: Id,
    name: String,
    price: f32,
}

impl_store_data!(Skull);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    id: Id,
    skull: Skull,
    amount: f32,
}

impl_store_data!(Quick);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    id: Id,
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
    fn create(&mut self, data: D) -> Result<Id, Error>;
    fn read(&self, id: Id) -> Result<&D, Error>;
    fn update(&mut self, id: Id, data: D) -> Result<D, Error>;
    fn delete(&mut self, id: Id) -> Result<D, Error>;
}

pub fn in_memory() -> in_memory::InMemory {
    in_memory::InMemory::default()
}
