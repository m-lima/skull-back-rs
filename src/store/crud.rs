use super::{Data, Error, Id, Occurrence, Quick, Skull};

pub trait Store: Send + Sync + std::panic::RefUnwindSafe + 'static {
    fn skull(&self, user: &str) -> Result<&std::sync::RwLock<dyn Crud<Skull>>, Error>;
    fn quick(&self, user: &str) -> Result<&std::sync::RwLock<dyn Crud<Quick>>, Error>;
    fn occurrence(&self, user: &str) -> Result<&std::sync::RwLock<dyn Crud<Occurrence>>, Error>;
}

// TODO: When using a RDB, will this interface still make sense?
// TODO: Is it possible to avoid the Vec's?
// TODO: OFfer a filter per day for Occurrence
pub trait Crud<D: Data> {
    fn list(&self, limit: Option<usize>) -> Result<Vec<std::borrow::Cow<'_, D::Id>>, Error>;
    fn filter_list(
        &self,
        filter: Box<dyn Fn(&D::Id) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, D::Id>>, Error>;
    fn create(&mut self, data: D) -> Result<Id, Error>;
    fn read(&self, id: Id) -> Result<std::borrow::Cow<'_, D::Id>, Error>;
    fn update(&mut self, id: Id, data: D) -> Result<D::Id, Error>;
    fn delete(&mut self, id: Id) -> Result<D::Id, Error>;
    fn last_modified(&self) -> Result<std::time::SystemTime, Error>;
}

pub trait Selector: Data {
    fn read<'a>(
        store: &'a dyn Store,
        user: &str,
    ) -> Result<std::sync::RwLockReadGuard<'a, (dyn Crud<Self> + 'static)>, Error>;

    fn write<'a>(
        store: &'a dyn Store,
        user: &str,
    ) -> Result<std::sync::RwLockWriteGuard<'a, (dyn Crud<Self> + 'static)>, Error>;
}

macro_rules! impl_selector {
    ($name:ty, $fn:ident) => {
        impl Selector for $name {
            fn read<'a>(
                store: &'a dyn Store,
                user: &str,
            ) -> Result<std::sync::RwLockReadGuard<'a, (dyn Crud<Self> + 'static)>, Error> {
                store
                    .$fn(user)?
                    .read()
                    .map_err(|_| Error::FailedToAcquireLock)
            }

            fn write<'a>(
                store: &'a dyn Store,
                user: &str,
            ) -> Result<std::sync::RwLockWriteGuard<'a, (dyn Crud<Self> + 'static)>, Error> {
                store
                    .$fn(user)?
                    .write()
                    .map_err(|_| Error::FailedToAcquireLock)
            }
        }
    };
}

impl_selector!(Skull, skull);
impl_selector!(Quick, quick);
impl_selector!(Occurrence, occurrence);
