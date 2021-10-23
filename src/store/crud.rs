use super::{Data, Error, Id, Occurrence, Quick, Skull, WithId};

pub trait Store: Send + 'static {
    fn skull(&mut self) -> &mut dyn Crud<Skull>;
    fn quick(&mut self) -> &mut dyn Crud<Quick>;
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence>;
}

// TODO: When using a RDB, will this interface still make sense?
// TODO: Is it possible to avoid the Vec's?
// TODO: OFfer a filter per day for Occurrence
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
    fn last_modified(&self, user: &str) -> Result<std::time::SystemTime, Error>;
}

pub trait Selector: Data {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self>;
}

impl Selector for Skull {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.skull()
    }
}

impl Selector for Quick {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.quick()
    }
}

impl Selector for Occurrence {
    fn select(store: &mut dyn Store) -> &mut dyn Crud<Self> {
        store.occurrence()
    }
}
