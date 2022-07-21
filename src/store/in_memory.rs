use super::{Crud, Data, Error, Id, Occurrence, Quick, Skull, Store, WithId};

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
    count: u32,
    data: Vec<D::Id>,
    last_modified: std::time::SystemTime,
}

impl<D: Data> Default for UserContainer<D> {
    fn default() -> Self {
        Self {
            count: 0,
            data: Vec::new(),
            last_modified: std::time::SystemTime::now(),
        }
    }
}

impl<D: Data> UserContainer<D> {
    fn id_to_index(&self, id: Id) -> Option<usize> {
        if self.data.is_empty() {
            None
        } else {
            let index = <usize as std::convert::TryFrom<Id>>::try_from(id).ok()?;
            Some(std::cmp::min(self.data.len() - 1, index))
        }
    }

    fn find(&self, id: Id) -> Option<usize> {
        for i in (0..=self.id_to_index(id)?).rev() {
            if self.data[i].id() == id {
                return Some(i);
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl<D: Data> Crud<D> for std::sync::RwLock<UserContainer<D>> {
    async fn list(&self, limit: Option<u32>) -> Result<Vec<D::Id>, Error> {
        let lock = self.read()?;
        Ok(lock
            .data
            .iter()
            .skip(
                lock.data.len()
                    - limit
                        .map(usize::try_from)
                        .and_then(Result::ok)
                        .unwrap_or(lock.data.len()),
            )
            .map(Clone::clone)
            .collect())
    }

    async fn create(&self, data: D) -> Result<Id, Error> {
        let mut lock = self.write()?;
        if lock.count == u32::MAX {
            return Err(Error::StoreFull);
        }
        lock.last_modified = std::time::SystemTime::now();
        let id = lock.count;
        let with_id = D::Id::new(id, data);
        lock.data.push(with_id);
        lock.count += 1;
        Ok(id)
    }

    async fn read(&self, id: Id) -> Result<D::Id, Error> {
        let lock = self.read()?;
        lock.find(id)
            .ok_or(Error::NotFound(id))
            .map(|i| &lock.data[i])
            .map(Clone::clone)
    }

    async fn update(&self, id: Id, data: D) -> Result<D::Id, Error> {
        let mut lock = self.write()?;
        lock.find(id).ok_or(Error::NotFound(id)).map(|i| {
            lock.last_modified = std::time::SystemTime::now();
            let old = &mut lock.data[i];
            let mut with_id = D::Id::new(old.id(), data);
            std::mem::swap(old, &mut with_id);
            with_id
        })
    }

    async fn delete(&self, id: Id) -> Result<D::Id, Error> {
        let mut lock = self.write()?;
        lock.find(id).ok_or(Error::NotFound(id)).map(|i| {
            lock.last_modified = std::time::SystemTime::now();
            lock.data.remove(i)
        })
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let lock = self.read()?;
        Ok(lock.last_modified)
    }
}

#[cfg(test)]
mod test {
    use crate::store::{Quick, Selector};

    use super::{Crud, Error, InMemory, Skull, UserContainer, WithId};

    type SkullId = <Skull as super::Data>::Id;

    const USER: &str = "bloink";

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

    fn new_skull(name: &str, unit_price: f32) -> Skull {
        Skull {
            name: String::from(name),
            color: String::from("red"),
            icon: String::new(),
            unit_price,
            limit: None,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fetches_user_container() {
        let mut store = InMemory::new(&[USER]);
        let skull = new_skull("skull", 0.4);
        let id = Skull::select(&store, USER)
            .unwrap()
            .create(skull)
            .await
            .unwrap();

        assert!(store.skull.data.len() == 1);
        assert!(
            store
                .skull
                .data
                .remove(USER)
                .unwrap()
                .into_inner()
                .unwrap()
                .data
                .len()
                == 1
        );
        assert!(id == 0);
    }

    #[test]
    fn reject_unknown_user() {
        let store = InMemory::new(&[USER]);
        assert_eq!(
            Skull::select(&store, "unknown")
                .map(|_| ())
                .unwrap_err()
                .to_string(),
            Error::NoSuchUser(String::from("unknown")).to_string()
        );
    }

    #[allow(clippy::too_many_lines)]
    #[tokio::test(flavor = "multi_thread")]
    async fn last_modified() {
        let mut store = InMemory::new(&[USER]);

        let last_modified = Skull::select(&store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap();

        // List [no change]
        Skull::select(&store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap();
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );

        // Create [change]
        Skull::select(&store, USER)
            .unwrap()
            .create(new_skull("bla", 1.0))
            .await
            .unwrap();
        assert_ne!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );
        let last_modified = Skull::select(&store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap();

        // Read [no change]
        Skull::select(&store, USER).unwrap().read(0).await.unwrap();
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );

        // Update [change]
        Skull::select(&store, USER)
            .unwrap()
            .update(0, new_skull("bla", 2.0))
            .await
            .unwrap();
        assert_ne!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );
        let last_modified = Skull::select(&store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap();

        // Delete [change]
        Skull::select(&store, USER)
            .unwrap()
            .delete(0)
            .await
            .unwrap();
        assert_ne!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );
        let last_modified = Skull::select(&store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap();

        // Create failure [no change]
        store
            .skull
            .data
            .get_mut(USER)
            .unwrap()
            .write()
            .unwrap()
            .count = u32::MAX;
        assert!(Skull::select(&store, USER)
            .unwrap()
            .create(new_skull("bla", 1.0))
            .await
            .is_err());
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );

        // Update failure [no change]
        assert!(Skull::select(&store, USER)
            .unwrap()
            .update(3, new_skull("bla", 1.0))
            .await
            .is_err());
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );

        // Delete failure [no change]
        assert!(Skull::select(&store, USER)
            .unwrap()
            .delete(5)
            .await
            .is_err());
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );

        // Stores don't affect each other
        Quick::select(&store, USER)
            .unwrap()
            .create(Quick {
                skull: 0,
                amount: 3.0,
            })
            .await
            .unwrap();
        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );
        assert_ne!(
            Quick::select(&store, USER)
                .unwrap()
                .last_modified()
                .await
                .unwrap(),
            last_modified
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list() {
        let store = InMemory::new(&[USER]);

        Skull::select(&store, USER)
            .unwrap()
            .create(new_skull("skull", 0.1))
            .await
            .unwrap();
        Skull::select(&store, USER)
            .unwrap()
            .create(new_skull("skull", 0.2))
            .await
            .unwrap();
        Skull::select(&store, USER)
            .unwrap()
            .create(new_skull("skull", 0.3))
            .await
            .unwrap();

        {
            let skulls = Skull::select(&store, USER)
                .unwrap()
                .list(None)
                .await
                .unwrap()
                .len();
            assert_eq!(skulls, 3);
        }
        {
            let skulls = Skull::select(&store, USER)
                .unwrap()
                .list(Some(1))
                .await
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(skulls, vec![SkullId::new(2, new_skull("skull", 0.3))]);
        }
        {
            let skulls = Skull::select(&store, USER)
                .unwrap()
                .list(Some(0))
                .await
                .unwrap()
                .len();
            assert_eq!(skulls, 0);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let skull = new_skull("skull", 0.4);
        let id = container.create(skull).await.unwrap();

        let container = container.read().unwrap();
        assert!(container.data.len() == 1);
        assert!(id == 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_store_full() {
        let container = std::sync::RwLock::new(UserContainer {
            count: u32::MAX,
            data: Vec::new(),
            last_modified: std::time::SystemTime::now(),
        });
        let skull = new_skull("skull", 0.4);

        assert_eq!(
            container.create(skull).await.unwrap_err().to_string(),
            Error::StoreFull.to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn read() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let expected = SkullId::new(3, new_skull("skull", 0.4));
        container.write().unwrap().data.push(expected.clone());

        assert_eq!(Crud::<Skull>::read(&container, 3).await.unwrap(), expected);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn read_not_found() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let id = 3;
        assert_eq!(
            Crud::<Skull>::read(&container, id)
                .await
                .unwrap_err()
                .to_string(),
            Error::NotFound(id).to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let old = SkullId::new(3, new_skull("skull", 0.4));
        let new = new_skull("bla", 0.7);
        let expected = SkullId::new(3, new.clone());
        container.write().unwrap().data.push(old.clone());

        assert_eq!(container.update(3, new).await.unwrap(), old);
        assert_eq!(container.read().unwrap().data[0], expected);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_not_found() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let new = new_skull("bla", 0.7);
        assert!(matches!(
            container.update(3, new).await,
            Err(Error::NotFound(3))
        ));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let container = std::sync::RwLock::new(UserContainer::<Skull>::default());
        let skull = SkullId::new(3, new_skull("skull", 0.4));
        container.write().unwrap().data.push(skull.clone());

        assert_eq!(container.delete(3).await.unwrap(), skull);
        assert!(container.read().unwrap().data.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_not_found() {
        let container = std::sync::RwLock::new(UserContainer::<Skull>::default());
        assert!(matches!(container.delete(3).await, Err(Error::NotFound(3))));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn id_always_grows() {
        let container = std::sync::RwLock::new(UserContainer::default());
        let skull = new_skull("skull", 0.4);

        let id = container.create(skull.clone()).await.unwrap();
        assert_eq!(id, 0);
        assert!(container.delete(id).await.is_ok());
        assert!(container.read().unwrap().data.is_empty());

        let id = container.create(skull).await.unwrap();
        assert_eq!(id, 1);
    }

    #[allow(clippy::cast_precision_loss)]
    #[tokio::test(flavor = "multi_thread")]
    async fn find() {
        let container = std::sync::RwLock::new(UserContainer::default());
        for i in 0..30 {
            container
                .create(new_skull("skull", i as f32))
                .await
                .unwrap();
        }

        container
            .write()
            .unwrap()
            .data
            .retain(|d| d.id() % 3 != 0 && d.id() % 4 != 0);

        for i in 0..30 {
            assert_eq!(
                Crud::<Skull>::read(&container, i).await.is_ok(),
                i % 3 != 0 && i % 4 != 0
            );
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[tokio::test(flavor = "multi_thread")]
    async fn delete_from_list() {
        let container = std::sync::RwLock::new(UserContainer::default());
        for i in 0..30 {
            container
                .create(new_skull("skull", i as f32))
                .await
                .unwrap();
        }

        let mut reference = container.read().unwrap().data.clone();

        reference.retain(|d| d.id() % 3 != 0 && d.id() % 4 != 0);

        for i in 0..30 {
            if i % 3 == 0 || i % 4 == 0 {
                container.delete(i).await.unwrap();
            }
        }

        assert_eq!(container.read().unwrap().data, reference);
    }
}
