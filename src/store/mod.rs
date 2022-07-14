mod crud;
mod data;
mod error;
// mod in_file;
mod in_memory;

pub type Id = u32;
pub use crud::{Crud, Selector, Store};
pub use data::{Data, Occurrence, Quick, Skull, WithId};
pub use error::Error;

pub fn in_memory<S, I>(users: I) -> impl Store
where
    S: ToString,
    I: std::iter::IntoIterator<Item = S>,
{
    in_memory::InMemory::new(users)
}

#[allow(clippy::unnecessary_wraps)]
// pub fn in_file<S, I, P>(path: P, users: I) -> anyhow::Result<impl Store>
pub fn in_file<S, I, P>(_path: P, users: I) -> anyhow::Result<impl Store>
where
    // S: AsRef<str>,
    S: ToString,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    Ok(in_memory::InMemory::new(users))
}
