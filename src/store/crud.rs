use super::{
    in_db::SqlData, in_file::FileData, in_memory::MemoryData, Data, Error, Id, Occurrence, Quick,
    Skull,
};

pub trait Store: Send + Sync + std::panic::RefUnwindSafe + 'static {
    type Crud<D: Selector>: Crud<D>;

    fn skull(&self, user: &str) -> Result<&Self::Crud<Skull>, Error>;
    fn quick(&self, user: &str) -> Result<&Self::Crud<Quick>, Error>;
    fn occurrence(&self, user: &str) -> Result<&Self::Crud<Occurrence>, Error>;
}

pub type Response<T> = Result<(T, std::time::SystemTime), Error>;

pub trait Selector: Data + SqlData + FileData + MemoryData {
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

pub struct SyncResponse<T>(Option<T>);

impl<T> SyncResponse<T> {
    pub fn new(data: T) -> Self {
        Self(Some(data))
    }
}

impl<T: Unpin + Send> std::future::Future for SyncResponse<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match std::pin::Pin::into_inner(self).0.take() {
            Some(data) => std::task::Poll::Ready(data),
            None => panic!("Double polled a ready Future"),
        }
    }
}

pub trait Crud<D: Data>: Send + Sync {
    type Future<T: Send + Unpin>: std::future::Future<Output = T> + Send;

    fn list(&self, limit: Option<u32>) -> Self::Future<Response<Vec<D::Id>>>;
    fn create(&self, data: D) -> Self::Future<Response<Id>>;
    fn read(&self, id: Id) -> Self::Future<Response<D::Id>>;
    fn update(&self, id: Id, data: D) -> Self::Future<Response<D::Id>>;
    fn delete(&self, id: Id) -> Self::Future<Response<D::Id>>;
    fn last_modified(&self) -> Self::Future<Result<std::time::SystemTime, Error>>;
}
