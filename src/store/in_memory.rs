use super::{crud::Response, Crud, Data, Error, Id, Occurrence, Quick, Skull, Store, WithId};

#[derive(Debug, Default)]
pub struct InMemory {
    skull: Container<Skull>,
    quick: Container<Quick>,
    occurrence: Container<Occurrence>,
}

impl InMemory {
    pub fn new<S, I>(users: I) -> Self
    where
        S: ToString,
        I: std::iter::IntoIterator<Item = S>,
    {
        let mut in_memory = InMemory::default();
        users.into_iter().for_each(|user| {
            in_memory.skull.data.insert(
                user.to_string(),
                std::sync::RwLock::new(UserContainer::default()),
            );
            in_memory.quick.data.insert(
                user.to_string(),
                std::sync::RwLock::new(UserContainer::default()),
            );
            in_memory.occurrence.data.insert(
                user.to_string(),
                std::sync::RwLock::new(UserContainer::default()),
            );
        });
        in_memory
    }
}

impl Store for InMemory {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error> {
        let user_container = self
            .skull
            .data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }

    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error> {
        let user_container = self
            .quick
            .data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }

    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error> {
        let user_container = self
            .occurrence
            .data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }
}

#[derive(Debug)]
pub(super) struct Container<D: Data> {
    data: std::collections::HashMap<String, std::sync::RwLock<UserContainer<D>>>,
}

impl<D: Data> Default for Container<D> {
    fn default() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub(super) struct UserContainer<D: Data> {
    next_id: u32,
    data: Vec<D::Id>,
    last_modified: std::time::SystemTime,
}

impl<D: Data> Default for UserContainer<D> {
    fn default() -> Self {
        Self {
            next_id: 1,
            data: Vec::new(),
            last_modified: std::time::SystemTime::now(),
        }
    }
}

impl<D: Data> UserContainer<D> {
    fn find(&self, id: Id) -> Option<usize> {
        self.data
            .iter()
            .take(usize::try_from(id).ok().map(|id| id.min(self.data.len()))?)
            .rposition(|d| d.id() == id)
    }
}

#[async_trait::async_trait]
impl<D: MemoryData> Crud<D> for std::sync::RwLock<UserContainer<D>> {
    async fn list(&self, limit: Option<u32>) -> Response<Vec<D::Id>> {
        let lock = self.read()?;
        let len = lock.data.len();
        Ok((
            lock.data
                .iter()
                .skip(
                    len - limit
                        .map(usize::try_from)
                        .and_then(Result::ok)
                        .unwrap_or(len)
                        .min(len),
                )
                .map(Clone::clone)
                .collect(),
            lock.last_modified,
        ))
    }

    async fn create(&self, data: D) -> Response<Id> {
        let mut lock = self.write()?;
        if lock.next_id == u32::MAX {
            return Err(Error::StoreFull);
        }
        if data.conflicts(lock.data.iter()) {
            return Err(Error::Constraint);
        }
        lock.last_modified = std::time::SystemTime::now();
        let id = lock.next_id;
        let with_id = D::Id::new(id, data);
        lock.data.push(with_id);
        lock.next_id += 1;
        Ok((id, lock.last_modified))
    }

    async fn read(&self, id: Id) -> Response<D::Id> {
        let lock = self.read()?;
        lock.find(id)
            .ok_or(Error::NotFound(id))
            .map(|i| &lock.data[i])
            .map(Clone::clone)
            .map(|data| (data, lock.last_modified))
    }

    async fn update(&self, id: Id, data: D) -> Response<D::Id> {
        let mut lock = self.write()?;
        if data.conflicts(lock.data.iter().filter(|d| d.id() != id)) {
            return Err(Error::Constraint);
        }
        lock.find(id).ok_or(Error::NotFound(id)).map(|i| {
            let old = &mut lock.data[i];
            let mut with_id = D::Id::new(old.id(), data);
            if old != &with_id {
                std::mem::swap(old, &mut with_id);
                lock.last_modified = std::time::SystemTime::now();
            }
            (with_id, lock.last_modified)
        })
    }

    async fn delete(&self, id: Id) -> Response<D::Id> {
        let mut lock = self.write()?;
        lock.find(id).ok_or(Error::NotFound(id)).map(|i| {
            lock.last_modified = std::time::SystemTime::now();
            (lock.data.remove(i), lock.last_modified)
        })
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let lock = self.read()?;
        Ok(lock.last_modified)
    }
}

trait MemoryData: Data {
    fn conflicts<'i>(&self, data: impl Iterator<Item = &'i Self::Id>) -> bool;
}

impl MemoryData for Skull {
    fn conflicts<'i>(&self, mut data: impl Iterator<Item = &'i Self::Id>) -> bool {
        data.any(|d| d.name == self.name || d.color == self.color || d.icon == self.icon)
    }
}

impl MemoryData for Quick {
    fn conflicts<'i>(&self, mut data: impl Iterator<Item = &'i Self::Id>) -> bool {
        data.any(|d| d.skull == self.skull && (d.amount - self.amount).abs() < f32::EPSILON)
    }
}

impl MemoryData for Occurrence {
    fn conflicts<'i>(&self, _: impl Iterator<Item = &'i Self::Id>) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::store::test::USER;

    use super::{Crud, Error, InMemory, Skull, UserContainer};

    crate::impl_crud_tests!(InMemory, InMemory::new(&[USER]));

    mod construction {
        use super::InMemory;

        #[test]
        fn direct_slice() {
            let store = InMemory::new(&["0", "1", "2"]);
            assert_eq!(store.skull.data.keys().len(), 3);
        }

        #[test]
        fn vec_str() {
            let store = InMemory::new(vec!["0", "1", "2"]);
            assert_eq!(store.skull.data.keys().len(), 3);
        }

        #[test]
        fn vec_string() {
            let store = InMemory::new(vec!["0".to_string(), "1".to_string(), "2".to_string()]);
            assert_eq!(store.skull.data.keys().len(), 3);
        }

        #[test]
        fn ref_vec_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(&v);
            assert_eq!(store.skull.data.keys().len(), 3);
        }

        #[test]
        fn slice_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(v.as_slice());
            assert_eq!(store.skull.data.keys().len(), 3);
        }

        #[test]
        fn iter_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(v.iter());
            assert_eq!(store.skull.data.keys().len(), 3);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_store_full() {
        let container = std::sync::RwLock::new(UserContainer {
            next_id: u32::MAX,
            data: Vec::new(),
            last_modified: std::time::SystemTime::now(),
        });

        let skull = Skull {
            name: String::from("skull"),
            color: String::from("red"),
            icon: String::new(),
            unit_price: 1.,
            limit: None,
        };

        assert_eq!(
            container.create(skull).await.unwrap_err().to_string(),
            Error::StoreFull.to_string()
        );
    }
}
