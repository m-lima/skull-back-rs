use super::{Data, Error, Id, Occurrence, Quick, Skull};

pub trait Store: Send + Sync + std::panic::RefUnwindSafe + 'static {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error>;
    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error>;
    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error>;
}

#[async_trait::async_trait]
pub trait Crud<D: Data>: Send + Sync {
    async fn list(&self, limit: Option<u32>) -> Result<Vec<D::Id>, Error>;
    async fn create(&self, data: D) -> Result<Id, Error>;
    async fn read(&self, id: Id) -> Result<D::Id, Error>;
    async fn update(&self, id: Id, data: D) -> Result<D::Id, Error>;
    async fn delete(&self, id: Id) -> Result<D::Id, Error>;
    async fn last_modified(&self) -> Result<std::time::SystemTime, Error>;
}

pub trait Selector: Data {
    fn select<'a>(store: &'a dyn Store, user: &str) -> Result<&'a dyn Crud<Self>, Error>;
}

macro_rules! impl_selector {
    ($name:ty, $fn:ident) => {
        impl Selector for $name {
            fn select<'a>(store: &'a dyn Store, user: &str) -> Result<&'a dyn Crud<Self>, Error> {
                store.$fn(user)
            }
        }
    };
}

impl_selector!(Skull, skull);
impl_selector!(Quick, quick);
impl_selector!(Occurrence, occurrence);
