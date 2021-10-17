// Allowed because of proc-macro
#![allow(clippy::trait_duplication_in_bounds)]

mod crud;
mod data;
mod error;
mod in_file;
mod in_memory;

pub type Id = u32;
pub use crud::{Crud, Selector, Store};
pub use data::{Data, LastModified, Occurrence, Quick, Skull, WithId};
pub use error::Error;

pub fn in_memory<S, I>(users: I) -> impl Store
where
    S: ToString,
    I: std::iter::IntoIterator<Item = S>,
{
    in_memory::InMemory::new(users)
}

pub fn in_file<S, I, P>(path: P, users: I) -> anyhow::Result<impl Store>
where
    S: AsRef<str>,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    in_file::InFile::new(path, users)
}
