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
            in_memory
                .skull
                .data
                .insert(user.to_string(), UserContainer::default());
            in_memory
                .quick
                .data
                .insert(user.to_string(), UserContainer::default());
            in_memory
                .occurrence
                .data
                .insert(user.to_string(), UserContainer::default());
        });
        in_memory
    }
}

impl Store for InMemory {
    fn skull(&mut self) -> &mut dyn Crud<Skull> {
        &mut self.skull
    }
    fn quick(&mut self) -> &mut dyn Crud<Quick> {
        &mut self.quick
    }
    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence> {
        &mut self.occurrence
    }
}

#[derive(Debug)]
pub(super) struct Container<D: Data> {
    data: std::collections::HashMap<String, UserContainer<D>>,
}

impl<D: Data> Default for Container<D> {
    fn default() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl<D: Data> Crud<D> for Container<D> {
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .map(UserContainer::list)
            .map(|s| s.map(std::borrow::Cow::Borrowed).collect())
    }

    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&WithId<D>) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .map(UserContainer::list)
            .map(|s| {
                s.filter(|d| (filter)(d))
                    .map(std::borrow::Cow::Borrowed)
                    .collect()
            })
    }

    fn create(&mut self, user: &str, data: D) -> Result<Id, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.create(data))
    }

    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, WithId<D>>, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.read(id))
            .map(std::borrow::Cow::Borrowed)
    }

    fn update(&mut self, user: &str, id: Id, data: D) -> Result<WithId<D>, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.update(id, data))
    }

    fn delete(&mut self, user: &str, id: Id) -> Result<WithId<D>, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.delete(id))
    }

    fn last_modified(&self, user: &str) -> Result<std::time::SystemTime, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .map(|store| store.last_modified)
    }
}

#[derive(Debug)]
pub(super) struct UserContainer<D: Data> {
    count: u32,
    data: Vec<WithId<D>>,
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
            if self.data[i].id == id {
                return Some(i);
            }
        }
        None
    }

    fn list(&self) -> impl std::iter::Iterator<Item = &WithId<D>> {
        self.data.iter()
    }

    fn create(&mut self, data: D) -> Result<Id, Error> {
        if self.count == u32::MAX {
            return Err(Error::StoreFull);
        }
        self.last_modified = std::time::SystemTime::now();
        let id = self.count;
        let with_id = WithId::new(id, data);
        self.data.push(with_id);
        self.count += 1;
        Ok(id)
    }

    fn read(&self, id: Id) -> Result<&WithId<D>, Error> {
        self.find(id)
            .ok_or(Error::NotFound(id))
            .map(|i| &self.data[i])
    }

    fn update(&mut self, id: Id, data: D) -> Result<WithId<D>, Error> {
        self.find(id).ok_or(Error::NotFound(id)).map(|i| {
            self.last_modified = std::time::SystemTime::now();
            let old = &mut self.data[i];
            let mut with_id = WithId::new(old.id, data);
            std::mem::swap(old, &mut with_id);
            with_id
        })
    }

    fn delete(&mut self, id: Id) -> Result<WithId<D>, Error> {
        self.find(id).ok_or(Error::NotFound(id)).map(|i| {
            self.last_modified = std::time::SystemTime::now();
            self.data.remove(i)
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Error, InMemory, Skull, Store, UserContainer, WithId};

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

    #[test]
    fn fetches_user_container() {
        let mut store = InMemory::new(&["bloink"]);
        let skull = new_skull("skull", 0.4);
        let id = store.skull().create("bloink", skull).unwrap();

        assert!(store.skull.data.get("bloink").unwrap().data.len() == 1);
        assert!(id == 0);
    }

    #[test]
    fn reject_unknown_user() {
        let mut store = InMemory::new(&["bloink"]);
        let skull = new_skull("skull", 0.4);
        assert_eq!(
            store
                .skull()
                .create("unknown", skull)
                .unwrap_err()
                .to_string(),
            Error::NoSuchUser(String::from("unknown")).to_string()
        );
    }

    #[test]
    fn last_modified() {
        const USER: &str = "bloink";
        let mut store = InMemory::new(&[USER]);

        assert_eq!(
            store
                .skull()
                .last_modified("unknown")
                .unwrap_err()
                .to_string(),
            Error::NoSuchUser(String::from("unknown")).to_string()
        );

        let mut last_modified = store.skull().last_modified(USER).unwrap();

        store.skull().list(USER).unwrap();
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        store.skull().filter_list(USER, Box::new(|_| true)).unwrap();
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        store.skull().create(USER, new_skull("bla", 1.0)).unwrap();
        assert_ne!(store.skull().last_modified(USER).unwrap(), last_modified);
        last_modified = store.skull().last_modified(USER).unwrap();

        store.skull().read(USER, 0).unwrap();
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        store
            .skull()
            .update(USER, 0, new_skull("bla", 2.0))
            .unwrap();
        assert_ne!(store.skull().last_modified(USER).unwrap(), last_modified);
        last_modified = store.skull().last_modified(USER).unwrap();

        store.skull().delete(USER, 0).unwrap();
        assert_ne!(store.skull().last_modified(USER).unwrap(), last_modified);
        last_modified = store.skull().last_modified(USER).unwrap();

        store.skull.data.get_mut(USER).unwrap().count = u32::MAX;
        assert!(store.skull().create(USER, new_skull("bla", 1.0)).is_err());
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        assert!(store
            .skull()
            .update(USER, 3, new_skull("bla", 1.0))
            .is_err());
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        assert!(store.skull().delete(USER, 5).is_err());
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);

        store
            .quick()
            .create(
                USER,
                super::Quick {
                    skull: 0,
                    amount: 3.0,
                },
            )
            .unwrap();
        assert_eq!(store.skull().last_modified(USER).unwrap(), last_modified);
        assert_ne!(store.quick().last_modified(USER).unwrap(), last_modified);
    }

    #[test]
    fn create() {
        let mut container = UserContainer::default();
        let skull = new_skull("skull", 0.4);
        let id = container.create(skull).unwrap();

        assert!(container.data.len() == 1);
        assert!(id == 0);
    }

    #[test]
    fn create_store_full() {
        let mut container = UserContainer {
            count: u32::MAX,
            data: Vec::new(),
            last_modified: std::time::SystemTime::now(),
        };
        let skull = new_skull("skull", 0.4);

        assert_eq!(
            container.create(skull).unwrap_err().to_string(),
            Error::StoreFull.to_string()
        );
    }

    #[test]
    fn read() {
        let mut container = UserContainer::default();
        let skull = WithId::new(3, new_skull("skull", 0.4));
        let expected = skull.clone();
        container.data.push(skull);

        assert_eq!(container.read(3).unwrap(), &expected);
    }

    #[test]
    fn read_not_found() {
        let container = UserContainer::<Skull>::default();
        let id = 3;
        assert_eq!(
            container.read(id).unwrap_err().to_string(),
            Error::NotFound(id).to_string()
        );
    }

    #[test]
    fn update() {
        let mut container = UserContainer::default();
        let old = WithId::new(3, new_skull("skull", 0.4));
        let new = new_skull("bla", 0.7);
        let expected = WithId::new(3, new.clone());
        container.data.push(old.clone());

        assert_eq!(container.update(3, new).unwrap(), old);
        assert_eq!(container.data[0], expected);
    }

    #[test]
    fn update_not_found() {
        let mut container = UserContainer::default();
        let new = new_skull("bla", 0.7);
        assert!(matches!(container.update(3, new), Err(Error::NotFound(3))));
    }

    #[test]
    fn delete() {
        let mut container = UserContainer::default();
        let skull = WithId::new(3, new_skull("skull", 0.4));
        container.data.push(skull.clone());

        assert_eq!(container.delete(3).unwrap(), skull);
        assert!(container.data.is_empty());
    }

    #[test]
    fn delete_not_found() {
        let mut container = UserContainer::<Skull>::default();
        assert!(matches!(container.delete(3), Err(Error::NotFound(3))));
    }

    #[test]
    fn id_always_grows() {
        let mut container = UserContainer::default();
        let skull = new_skull("skull", 0.4);

        let mut id = container.create(skull.clone()).unwrap();
        assert_eq!(id, 0);
        assert!(container.delete(id).is_ok());
        assert!(container.data.is_empty());

        id = container.create(skull).unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn find() {
        let mut container = UserContainer::default();
        for i in 0..30 {
            container.create(new_skull("skull", i as f32)).unwrap();
        }

        container.data.retain(|d| d.id % 3 != 0 && d.id % 4 != 0);

        for i in 0..30 {
            assert_eq!(container.read(i).is_ok(), i % 3 != 0 && i % 4 != 0);
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn delete_from_list() {
        let mut container = UserContainer::default();
        for i in 0..30 {
            container.create(new_skull("skull", i as f32)).unwrap();
        }

        let mut reference = container.data.clone();

        reference.retain(|d| d.id % 3 != 0 && d.id % 4 != 0);

        for i in 0..30 {
            if i % 3 == 0 || i % 4 == 0 {
                container.delete(i).unwrap();
            }
        }

        assert_eq!(container.data, reference);
    }
}
