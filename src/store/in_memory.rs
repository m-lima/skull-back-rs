use super::{Crud, Data, Error, Id, Occurrence, Quick, Skull, Store};

#[derive(Debug, Default)]
pub struct InMemory {
    skull: Container<Skull>,
    quick: Container<Quick>,
    occurrence: Container<Occurrence>,
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
    fn list(&self, user: &str) -> Result<Vec<(&Id, &D)>, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .map(UserContainer::list)
    }

    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&D) -> bool>,
    ) -> Result<Vec<(&Id, &D)>, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .map(|store| store.filter_list(&filter))
    }

    fn create(&mut self, user: &str, data: D) -> Result<Id, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.create(data))
    }

    fn read(&self, user: &str, id: Id) -> Result<&D, Error> {
        self.data
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.read(id))
    }

    fn update(&mut self, user: &str, id: Id, data: D) -> Result<D, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.update(id, data))
    }

    fn delete(&mut self, user: &str, id: Id) -> Result<D, Error> {
        self.data
            .get_mut(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
            .and_then(|store| store.delete(id))
    }
}

#[derive(Debug)]
pub(super) struct UserContainer<D: Data> {
    count: u32,
    data: std::collections::HashMap<Id, D>,
}

impl<D: Data> Default for UserContainer<D> {
    fn default() -> Self {
        Self {
            count: 0,
            data: std::collections::HashMap::new(),
        }
    }
}

impl<D: Data> UserContainer<D> {
    fn list(&self) -> Vec<(&Id, &D)> {
        self.data.iter().collect()
    }

    fn filter_list(&self, filter: &dyn Fn(&D) -> bool) -> Vec<(&Id, &D)> {
        self.data.iter().filter(|d| (filter)(d.1)).collect()
    }

    fn create(&mut self, data: D) -> Result<Id, Error> {
        if self.count == u32::MAX {
            return Err(Error::StoreFull);
        }
        let id = self.count;
        self.count += 1;
        self.data.insert(id, data);
        Ok(id)
    }

    fn read(&self, id: Id) -> Result<&D, Error> {
        self.data.get(&id).ok_or(Error::NotFound(id))
    }

    fn update(&mut self, id: Id, data: D) -> Result<D, Error> {
        self.data.insert(id, data).ok_or(Error::NotFound(id))
    }

    fn delete(&mut self, id: Id) -> Result<D, Error> {
        self.data.remove(&id).ok_or(Error::NotFound(id))
    }
}

#[cfg(test)]
mod test {
    use super::{Error, InMemory, Skull, Store};

    fn new_skull(name: &str, unit_price: f32) -> Skull {
        Skull {
            name: String::from(name),
            color: 0,
            icon: String::new(),
            unit_price,
            limit: None,
        }
    }

    // fn new_store(user: &str) -> InMemory {
    //     let mut store = InMemory::default();
    //     store.
    // }

    #[test]
    fn create() {
        let mut store = InMemory::default();
        let skull = new_skull("skull", 0.4);
        let id = store.skull().create(skull).unwrap();

        assert!(store.skull.data.len() == 1);
        assert!(id == 0);
    }

    #[test]
    fn create_store_full() {
        let mut store = InMemory::default();
        store.skull.count = u32::MAX;
        let skull = new_skull("skull", 0.4);

        assert_eq!(
            store.skull().create(skull).unwrap_err().to_string(),
            Error::StoreFull.to_string()
        );
    }

    #[test]
    fn read() {
        let mut store = InMemory::default();
        let id = 3;
        let skull = new_skull("skull", 0.4);
        let expected = skull.clone();
        store.skull.data.insert(id, skull);

        assert_eq!(store.skull().read(id).unwrap(), &expected);
    }

    #[test]
    fn read_not_found() {
        let mut store = InMemory::default();
        let id = 3;
        assert_eq!(
            store.skull().read(id).unwrap_err().to_string(),
            Error::NotFound(id).to_string()
        );
    }

    #[test]
    fn update() {
        let mut store = InMemory::default();
        let id = 3;
        let old = new_skull("skull", 0.4);
        let old_expected = old.clone();
        let new = new_skull("bla", 0.7);
        let new_expected = new.clone();
        store.skull.data.insert(id, old);

        assert_eq!(store.skull().update(id, new).unwrap(), old_expected);
        assert_eq!(store.skull.data.get(&id).unwrap(), &new_expected);
    }

    #[test]
    fn update_not_found() {
        let mut store = InMemory::default();
        let id = 3;
        let new = new_skull("bla", 0.7);
        assert_eq!(
            store.skull().update(id, new).unwrap_err().to_string(),
            Error::NotFound(id).to_string()
        );
    }

    #[test]
    fn delete() {
        let mut store = InMemory::default();
        let id = 3;
        let skull = new_skull("skull", 0.4);
        let expected = skull.clone();
        store.skull.data.insert(id, skull);

        assert_eq!(store.skull().delete(id).unwrap(), expected);
        assert!(store.skull.data.is_empty());
    }

    #[test]
    fn delete_not_found() {
        let mut store = InMemory::default();
        let id = 3;
        assert_eq!(
            store.skull().delete(id).unwrap_err().to_string(),
            Error::NotFound(id).to_string()
        );
    }

    #[test]
    fn id_always_grows() {
        let mut store = InMemory::default();
        let skull = new_skull("skull", 0.4);

        let mut id = store.skull().create(skull.clone()).unwrap();
        assert_eq!(id, 0);
        assert!(store.skull().delete(id).is_ok());
        assert!(store.skull.data.is_empty());

        id = store.skull().create(skull).unwrap();
        assert_eq!(id, 1);
    }
}
