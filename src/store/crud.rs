use super::{Data, Error, Id, Occurrence, Quick, Skull};

pub trait Store: Send + Sync + std::panic::RefUnwindSafe + 'static {
    type Crud<D: Selector>: Crud<D>;

    fn skull(&self, user: &str) -> Result<&Self::Crud<Skull>, Error>;
    fn quick(&self, user: &str) -> Result<&Self::Crud<Quick>, Error>;
    fn occurrence(&self, user: &str) -> Result<&Self::Crud<Occurrence>, Error>;
}

pub type Response<T> = Result<(T, std::time::SystemTime), Error>;

#[async_trait::async_trait]
pub trait Crud<D: Data>: Send + Sync {
    async fn list(&self, limit: Option<u32>) -> Response<Vec<D::Id>>;
    async fn create(&self, data: D) -> Response<Id>;
    async fn read(&self, id: Id) -> Response<D::Id>;
    async fn update(&self, id: Id, data: D) -> Response<D::Id>;
    async fn delete(&self, id: Id) -> Response<D::Id>;
    async fn last_modified(&self) -> Result<std::time::SystemTime, Error>;
}

pub trait Selector:
    Data + super::in_db::SqlData + super::in_file::FileData + super::in_memory::MemoryData
{
    fn select<'a, S: Store>(store: &'a S, user: &str) -> Result<&'a S::Crud<Self>, Error>;
}

macro_rules! impl_selector {
    ($name:ty, $fn:ident) => {
        impl Selector for $name {
            fn select<'a, S: Store>(store: &'a S, user: &str) -> Result<&'a S::Crud<Self>, Error> {
                store.$fn(user)
            }
        }
    };
}

impl_selector!(Skull, skull);
impl_selector!(Quick, quick);
impl_selector!(Occurrence, occurrence);
