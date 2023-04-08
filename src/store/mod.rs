mod crud;
mod data;
mod error;
mod in_db;
mod in_file;
mod in_memory;

#[cfg(all(test, nightly))]
mod bench;
#[cfg(test)]
mod test;

pub use crud::Crud;
pub use data::Id;
pub use error::Error;

use data::{Data, Occurrence, Quick, Skull, WithId};

pub trait Store: Send + Sync + std::panic::RefUnwindSafe + 'static {
    type Crud<M: Model>: Crud<M>;

    fn skull(&self, user: &str) -> Result<&Self::Crud<Skull>, Error>;
    fn quick(&self, user: &str) -> Result<&Self::Crud<Quick>, Error>;
    fn occurrence(&self, user: &str) -> Result<&Self::Crud<Occurrence>, Error>;
}

pub trait Model: Data + in_db::SqlData + in_file::FileData + in_memory::MemoryData {
    fn name() -> &'static str;
    fn select<'a, S: Store>(store: &'a S, user: &str) -> Result<&'a S::Crud<Self>, Error>;
}

macro_rules! impl_model {
    ($name: ty, $fn: ident) => {
        impl Model for $name {
            fn name() -> &'static str {
                stringify!($fn)
            }

            fn select<'a, S: Store>(store: &'a S, user: &str) -> Result<&'a S::Crud<Self>, Error> {
                store.$fn(user)
            }
        }
    };
}

impl_model!(Skull, skull);
impl_model!(Quick, quick);
impl_model!(Occurrence, occurrence);

pub const MODELS: (
    std::marker::PhantomData<Skull>,
    std::marker::PhantomData<Quick>,
    std::marker::PhantomData<Occurrence>,
) = (
    std::marker::PhantomData,
    std::marker::PhantomData,
    std::marker::PhantomData,
);

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
    in_file::InFile::new(gather_users(path, users)?)
}

pub fn in_db<S, I, P>(path: P, users: I) -> anyhow::Result<impl Store>
where
    S: AsRef<str>,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    in_db::InDb::new(gather_users(path, users)?)
}

fn gather_users<S, I, P>(
    path: P,
    users: I,
) -> anyhow::Result<std::collections::HashMap<String, std::path::PathBuf>>
where
    S: AsRef<str>,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    fn open_dir(path: &std::path::PathBuf) -> anyhow::Result<std::fs::ReadDir> {
        if !path.exists() {
            anyhow::bail!(
                "Store directory does not exist: {}",
                std::fs::canonicalize(path)
                    .unwrap_or_else(|_| path.clone())
                    .display()
            );
        }

        if !path.is_dir() {
            anyhow::bail!(
                "Store path is not a directory: {}",
                std::fs::canonicalize(path)
                    .unwrap_or_else(|_| path.clone())
                    .display()
            );
        }

        path.read_dir()
            .map_err(|e| anyhow::anyhow!("Store directory cannot be read: {e}"))
    }
    let path = std::path::PathBuf::from(path.as_ref());
    let open_dir = open_dir(&path)?;

    Ok(users
        .into_iter()
        .map(|user| path.join(user.as_ref()))
        .chain(open_dir.filter_map(Result::ok).map(|child| child.path()))
        .filter_map(|root| {
            root.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
                .map(|name| (name, root))
        })
        .collect())
}
