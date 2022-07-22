#[cfg(all(test, nightly))]
mod bench;
mod crud;
mod data;
mod error;
mod in_db;
mod in_file;
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

fn open_dir(path: &std::path::PathBuf) -> anyhow::Result<std::fs::ReadDir> {
    if !path.exists() {
        anyhow::bail!(
            "Store directory does not exist: {}",
            std::fs::canonicalize(&path)
                .unwrap_or_else(|_| path.clone())
                .display()
        );
    }

    if !path.is_dir() {
        anyhow::bail!(
            "Store path is not a directory: {}",
            std::fs::canonicalize(&path)
                .unwrap_or_else(|_| path.clone())
                .display()
        );
    }

    path.read_dir()
        .map_err(|e| anyhow::anyhow!("Store directory cannot be read: {e}"))
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

#[cfg(test)]
mod test_util {
    use crate::test_util::Assertion;

    use super::{Crud, Skull};

    pub async fn last_modified_eq<C: Crud<Skull>>(
        crud: &C,
        previous: std::time::SystemTime,
        op_time: impl Into<Option<std::time::SystemTime>>,
    ) -> Assertion<std::time::SystemTime> {
        let check_time = crud.last_modified().await.unwrap();

        if let Some(op_time) = op_time.into() {
            if check_time != op_time {
                return Assertion::err_ne(
                    "Returned last_modified vs API last_modified mismatch",
                    check_time,
                    op_time,
                );
            }
        }

        if check_time == previous {
            Assertion::Ok(check_time)
        } else {
            Assertion::err_ne(
                "Unwated modification to last_modified",
                check_time,
                previous,
            )
        }
    }

    pub async fn last_modified_ne<C: Crud<Skull>>(
        crud: &C,
        previous: std::time::SystemTime,
        op_time: impl Into<Option<std::time::SystemTime>>,
    ) -> Assertion<std::time::SystemTime> {
        let check_time = crud.last_modified().await.unwrap();

        if let Some(op_time) = op_time.into() {
            if check_time != op_time {
                return Assertion::err_ne(
                    "Returned last_modified vs API last_modified mismatch",
                    check_time,
                    op_time,
                );
            }
        }

        if check_time == previous {
            Assertion::err_eq("Expected modification to last_modified", check_time)
        } else {
            Assertion::Ok(check_time)
        }
    }
}
