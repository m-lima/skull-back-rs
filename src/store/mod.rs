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

pub trait Data: serde::Serialize + serde::de::DeserializeOwned {}

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

pub trait Store: Send + 'static {
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

// TODO: When using a RDB, will this interface still make sense?
// TODO: Is it possible to avoid the Vec's?
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

pub trait CrudSelector<D: Data> {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<D>;
}

struct SkullSelector;
impl CrudSelector<Skull> for SkullSelector {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Skull> {
        store.skull()
    }
}

struct QuickSelector;
impl CrudSelector<Quick> for QuickSelector {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Quick> {
        store.quick()
    }
}

struct OccurrenceSelector;
impl CrudSelector<Occurrence> for OccurrenceSelector {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Occurrence> {
        store.occurrence()
    }
}
