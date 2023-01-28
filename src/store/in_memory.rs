use super::{crud::Response, Crud, Data, Error, Id, Occurrence, Quick, Skull, Store, WithId};

#[derive(Debug, Default)]
pub struct InMemory {
    users: std::collections::HashMap<String, UserStore>,
}

impl InMemory {
    pub fn new<S, I>(users: I) -> Self
    where
        S: ToString,
        I: std::iter::IntoIterator<Item = S>,
    {
        let mut in_memory = InMemory::default();
        users.into_iter().for_each(|user| {
            in_memory.users.insert(
                user.to_string(),
                UserStore {
                    skull: std::sync::RwLock::new(UserContainer::default()),
                    quick: std::sync::RwLock::new(UserContainer::default()),
                    occurrence: std::sync::RwLock::new(UserContainer::default()),
                },
            );
        });
        in_memory
    }
}

impl Store for InMemory {
    type Crud<D: super::Selector> = UserStore;

    fn skull(&self, user: &str) -> Result<&Self::Crud<Skull>, Error> {
        let user_container = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }

    fn quick(&self, user: &str) -> Result<&Self::Crud<Quick>, Error> {
        let user_container = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }

    fn occurrence(&self, user: &str) -> Result<&Self::Crud<Occurrence>, Error> {
        let user_container = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_container)
    }
}

#[derive(Debug)]
pub struct UserStore {
    skull: std::sync::RwLock<UserContainer<Skull>>,
    quick: std::sync::RwLock<UserContainer<Quick>>,
    occurrence: std::sync::RwLock<UserContainer<Occurrence>>,
}

#[async_trait::async_trait]
impl<D: MemoryData> Crud<D> for UserStore {
    async fn list(&self, limit: Option<u32>) -> Response<Vec<D::Id>> {
        D::list(self, limit)
    }

    async fn create(&self, data: D) -> Response<Id> {
        D::create(self, data)
    }

    async fn read(&self, id: Id) -> Response<D::Id> {
        D::read(self, id)
    }

    async fn update(&self, id: Id, data: D) -> Response<D::Id> {
        D::update(self, id, data)
    }

    async fn delete(&self, id: Id) -> Response<D::Id> {
        D::delete(self, id)
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        D::last_modified(self)
    }
}

#[derive(Debug)]
pub struct UserContainer<D: Data> {
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

pub trait MemoryData: Data + 'static {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserContainer<Self>>;
    fn list(store: &UserStore, limit: Option<u32>) -> Response<Vec<Self::Id>>;
    fn create(store: &UserStore, data: Self) -> Response<Id>;
    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id>;
    fn delete(store: &UserStore, id: Id) -> Response<Self::Id>;
    fn conflicts(&self, other: &Self::Id) -> bool;

    fn read(store: &UserStore, id: Id) -> Response<Self::Id> {
        let lock = Self::as_read(store)?;
        lock.find(id)
            .ok_or(Error::NotFound(id))
            .map(|i| &lock.data[i])
            .map(Clone::clone)
            .map(|data| (data, lock.last_modified))
    }

    fn last_modified(store: &UserStore) -> Result<std::time::SystemTime, Error> {
        Ok(Self::as_read(store)?.last_modified)
    }

    fn list_inner(
        lock: &std::sync::RwLockReadGuard<'_, UserContainer<Self>>,
        limit: Option<u32>,
    ) -> Response<Vec<Self::Id>> {
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

    fn create_inner(
        lock: &mut std::sync::RwLockWriteGuard<'_, UserContainer<Self>>,
        data: Self,
    ) -> Response<Id> {
        if lock.next_id == u32::MAX {
            return Err(Error::StoreFull);
        }
        lock.last_modified = std::time::SystemTime::now();
        let id = lock.next_id;
        let with_id = Self::Id::new(id, data);
        lock.data.push(with_id);
        lock.next_id += 1;
        Ok((id, lock.last_modified))
    }

    fn update_inner(
        lock: &mut std::sync::RwLockWriteGuard<'_, UserContainer<Self>>,
        idx: usize,
        data: Self,
    ) -> Response<Self::Id> {
        let old = &mut lock.data[idx];
        let mut with_id = Self::Id::new(old.id(), data);
        if old != &with_id {
            std::mem::swap(old, &mut with_id);
            lock.last_modified = std::time::SystemTime::now();
        }
        Ok((with_id, lock.last_modified))
    }

    fn delete_inner(
        lock: &mut std::sync::RwLockWriteGuard<'_, UserContainer<Self>>,
        id: Id,
    ) -> Response<Self::Id> {
        let response = lock.find(id).ok_or(Error::NotFound(id)).map(|i| {
            lock.last_modified = std::time::SystemTime::now();
            (lock.data.remove(i), lock.last_modified)
        })?;
        lock.next_id = lock.data.last().map_or(0, WithId::id) + 1;
        Ok(response)
    }

    fn find_index(
        lock: &std::sync::RwLockWriteGuard<'_, UserContainer<Self>>,
        id: Id,
    ) -> Result<usize, Error> {
        lock.find(id).ok_or(Error::NotFound(id))
    }

    fn has_skull(
        lock: &std::sync::RwLockReadGuard<'_, UserContainer<Skull>>,
        skull: Id,
    ) -> Result<(), Error> {
        if lock.data.iter().any(|d| d.id() == skull) {
            Ok(())
        } else {
            Err(Error::Constraint)
        }
    }

    fn as_read(
        store: &UserStore,
    ) -> Result<std::sync::RwLockReadGuard<'_, UserContainer<Self>>, Error> {
        Ok(Self::get(store).read()?)
    }

    fn as_write(
        store: &UserStore,
    ) -> Result<std::sync::RwLockWriteGuard<'_, UserContainer<Self>>, Error> {
        Ok(Self::get(store).write()?)
    }
}

impl MemoryData for Skull {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserContainer<Self>> {
        &store.skull
    }

    fn list(store: &UserStore, limit: Option<u32>) -> Response<Vec<Self::Id>> {
        let lock = Self::as_read(store)?;
        Self::list_inner(&lock, limit)
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        let mut lock = Self::as_write(store)?;
        if lock.data.iter().any(|d| data.conflicts(d)) {
            Err(Error::Conflict)
        } else {
            Self::create_inner(&mut lock, data)
        }
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let mut lock = Self::as_write(store)?;
        let idx = Self::find_index(&lock, id)?;

        if lock
            .data
            .iter()
            .filter(|d| d.id != id)
            .any(|d| data.conflicts(d))
        {
            Err(Error::Conflict)
        } else {
            Self::update_inner(&mut lock, idx, data)
        }
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        let mut lock = Self::as_write(store)?;
        let occurrence_lock = Occurrence::as_read(store)?;

        if occurrence_lock.data.iter().any(|d| d.skull == id) {
            Err(Error::Constraint)
        } else {
            let mut quick_lock = Quick::as_write(store)?;
            let response = Self::delete_inner(&mut lock, id)?;
            quick_lock.data.retain(|d| d.skull != id);
            quick_lock.last_modified = std::time::SystemTime::now();
            Ok(response)
        }
    }

    fn conflicts(&self, other: &Self::Id) -> bool {
        self.name == other.name || self.color == other.color || self.icon == other.icon
    }
}

impl MemoryData for Quick {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserContainer<Self>> {
        &store.quick
    }

    fn list(store: &UserStore, limit: Option<u32>) -> Response<Vec<Self::Id>> {
        let lock = Self::as_read(store)?;
        Self::list_inner(&lock, limit)
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        let skull_lock = Skull::as_read(store)?;
        let mut lock = Self::as_write(store)?;

        Self::has_skull(&skull_lock, data.skull)?;

        if lock.data.iter().any(|d| data.conflicts(d)) {
            Err(Error::Conflict)
        } else {
            Self::create_inner(&mut lock, data)
        }
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let skull_lock = Skull::as_read(store)?;
        let mut lock = Self::as_write(store)?;

        let idx = Self::find_index(&lock, id)?;
        Self::has_skull(&skull_lock, data.skull)?;

        if lock
            .data
            .iter()
            .filter(|d| d.id != id)
            .any(|d| data.conflicts(d))
        {
            Err(Error::Conflict)
        } else {
            Self::update_inner(&mut lock, idx, data)
        }
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        let mut lock = Self::as_write(store)?;
        Self::delete_inner(&mut lock, id)
    }

    fn conflicts(&self, other: &Self::Id) -> bool {
        self.skull == other.skull && (self.amount - other.amount).abs() < f32::EPSILON
    }
}

#[async_trait::async_trait]
impl MemoryData for Occurrence {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserContainer<Self>> {
        &store.occurrence
    }

    fn list(store: &UserStore, limit: Option<u32>) -> Response<Vec<Self::Id>> {
        let (mut occurrences, last_modified) = {
            let lock = Self::as_read(store)?;
            Self::list_inner(&lock, None)?
        };
        occurrences.sort_unstable_by(|a, b| match b.millis.cmp(&a.millis) {
            std::cmp::Ordering::Equal => b.id.cmp(&a.id),
            c => c,
        });
        if let Some(limit) = limit {
            let limit = usize::try_from(limit).unwrap_or(occurrences.len());
            occurrences = occurrences.into_iter().take(limit).collect();
        }
        Ok((occurrences, last_modified))
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        let skull_lock = Skull::as_read(store)?;
        let mut lock = Self::as_write(store)?;

        Self::has_skull(&skull_lock, data.skull)?;
        Self::create_inner(&mut lock, data)
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let skull_lock = Skull::as_read(store)?;
        let mut lock = Self::as_write(store)?;

        let idx = Self::find_index(&lock, id)?;

        Self::has_skull(&skull_lock, data.skull)?;
        Self::update_inner(&mut lock, idx, data)
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        let mut lock = Self::as_write(store)?;
        Self::delete_inner(&mut lock, id)
    }

    fn conflicts(&self, _other: &Self::Id) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::store::test::USER;

    use super::{Error, InMemory, MemoryData, Skull, UserContainer};

    crate::impl_crud_tests!(InMemory, InMemory::new([USER]));

    mod construction {
        use super::InMemory;

        #[test]
        fn direct_slice() {
            let store = InMemory::new(["0", "1", "2"]);
            assert_eq!(store.users.keys().len(), 3);
        }

        #[test]
        fn vec_str() {
            let store = InMemory::new(vec!["0", "1", "2"]);
            assert_eq!(store.users.keys().len(), 3);
        }

        #[test]
        fn vec_string() {
            let store = InMemory::new(vec!["0".to_string(), "1".to_string(), "2".to_string()]);
            assert_eq!(store.users.keys().len(), 3);
        }

        #[test]
        fn ref_vec_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(v);
            assert_eq!(store.users.keys().len(), 3);
        }

        #[test]
        fn slice_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(v.as_slice());
            assert_eq!(store.users.keys().len(), 3);
        }

        #[test]
        fn iter_str() {
            let v = vec!["0", "1", "2"];
            let store = InMemory::new(v.iter());
            assert_eq!(store.users.keys().len(), 3);
        }
    }

    #[test]
    fn create_store_full() {
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
            Skull::create_inner(&mut container.write().unwrap(), skull)
                .unwrap_err()
                .to_string(),
            Error::StoreFull.to_string()
        );
    }
}
