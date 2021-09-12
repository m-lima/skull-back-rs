mod in_memory;

pub type Id = u32;

// TODO: Should this be a String and let the front end parse it?
pub type Color = u32;

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
    color: Color,
    icon: String,
    unit_price: f32,
    limit: Option<f32>,
}

impl Data for Skull {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Quick {
    skull: Skull,
    amount: f32,
}

impl Data for Quick {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Occurrence {
    skull: Skull,
    amount: f32,
    time: std::time::SystemTime,
}

impl Data for Occurrence {}

// TODO: This is a usuful class. But mostly for mapping output. Maybe doesn't belong here
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct DataWithId<'a, D: Data> {
    id: Id,
    #[serde(flatten)]
    data: &'a D,
}

impl<'a, D: Data> DataWithId<'a, D> {
    fn build(pair: &(&'a Id, &'a D)) -> Self {
        Self {
            id: *pair.0,
            data: pair.1,
        }
    }
}

pub trait Store: Send + 'static {
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

// TODO: When using a RDB, will this interface still make sense?
pub trait Crud<D: Data> {
    fn list(&self) -> Result<Vec<DataWithId<'_, D>>, Error>;
    fn filter_list(&self, filter: Box<dyn Fn(&D) -> bool>)
        -> Result<Vec<DataWithId<'_, D>>, Error>;
    fn create(&mut self, data: D) -> Result<Id, Error>;
    fn read(&self, id: Id) -> Result<&D, Error>;
    fn update(&mut self, id: Id, data: D) -> Result<D, Error>;
    fn delete(&mut self, id: Id) -> Result<D, Error>;
}

pub fn in_memory() -> in_memory::InMemory {
    in_memory::InMemory::default()
}
