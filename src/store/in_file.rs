use super::{Crud, Data, Error, Id, LastModified, Occurrence, Quick, Skull, Store};

pub struct InFile {
    path: std::path::PathBuf,
    users: std::collections::HashSet<String>,
}

impl InFile {
    pub fn new<S, I, P>(path: P, users: I) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
        I: std::iter::IntoIterator<Item = S>,
        P: AsRef<std::path::Path>,
    {
        let path = std::path::PathBuf::from(path.as_ref());

        if !path.exists() {
            anyhow::bail!(
                "Store directory does not exist: {}",
                std::fs::canonicalize(&path).unwrap_or(path).display()
            );
        }

        if !path.is_dir() {
            anyhow::bail!(
                "Store path is not a directory: {}",
                std::fs::canonicalize(&path).unwrap_or(path).display()
            );
        }

        let dir_reader = path
            .read_dir()
            .map_err(|e| anyhow::anyhow!("Store directory cannot be read: {}", e))?;

        let users = users
            .into_iter()
            .map(|user| String::from(user.as_ref()))
            .chain(
                dir_reader
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_dir())
                    .filter_map(|dir| dir.file_name().to_str().map(String::from)),
            )
            .collect::<std::collections::HashSet<_>>();

        for user in &users {
            let path = path.join(user);
            if !path.exists() {
                log::debug!("Creating {}", path.display());
                std::fs::create_dir(&path).map_err(|e| {
                    anyhow::anyhow!("Could not create user directory {}: {}", path.display(), e)
                })?;
            } else if !path.is_dir() {
                anyhow::bail!("User path is not a directory {}", path.display());
            }

            for file in
                [Skull::name(), Quick::name(), Occurrence::name()].map(|name| path.join(name))
            {
                if !file.exists() {
                    log::debug!("Creating {}", path.display());
                    std::fs::File::create(&file).map_err(|e| {
                        anyhow::anyhow!("Could not create {}: {}", file.display(), e)
                    })?;
                } else if file.is_dir() {
                    anyhow::bail!("Path {} is not a file", file.display());
                }
            }
            log::info!("Allowing {}", user);
        }

        Ok(Self { path, users })
    }
}

impl Store for InFile {
    fn last_modified(&self, user: &str) -> Result<LastModified, Error> {
        let user = fs::User::new(user, self)?;
        let skull = std::fs::metadata(user.to_path::<Skull>()).and_then(|f| f.modified())?;
        let quick = std::fs::metadata(user.to_path::<Quick>()).and_then(|f| f.modified())?;
        let occurrence =
            std::fs::metadata(user.to_path::<Occurrence>()).and_then(|f| f.modified())?;

        let timestamp = std::cmp::max(skull, std::cmp::max(quick, occurrence));
        Ok(LastModified { timestamp })
    }

    fn skull(&mut self) -> &mut dyn Crud<Skull> {
        self
    }

    fn quick(&mut self) -> &mut dyn Crud<Quick> {
        self
    }

    fn occurrence(&mut self) -> &mut dyn Crud<Occurrence> {
        self
    }
}

impl<D: Named> Crud<D> for InFile {
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, D>>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .map(|mut reader| {
                reader
                    .deserialize()
                    .filter_map(Result::ok)
                    .map(std::borrow::Cow::Owned)
                    .collect()
            })
    }

    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&D) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, D>>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .map(|mut reader| {
                reader
                    .deserialize()
                    .filter_map(Result::ok)
                    .filter(|d| (filter)(d))
                    .map(std::borrow::Cow::Owned)
                    .collect()
            })
    }

    fn create(&mut self, user: &str, mut data: D) -> Result<Id, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::reader(&user)
            .map(|mut reader| {
                reader
                    .deserialize::<D>()
                    .filter_map(Result::ok)
                    .last()
                    .map_or(0, |d| d.id() + 1)
            })
            .and_then(|id| {
                data.set_id(id);
                fs::append(user, data)?;
                Ok(id)
            })
    }

    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, D>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .and_then(|mut reader| {
                reader
                    .deserialize()
                    .filter_map(Result::ok)
                    .find(|d| Data::id(d) == id)
                    .map(std::borrow::Cow::Owned)
                    .ok_or(Error::NotFound(id))
            })
    }

    fn update(&mut self, user: &str, id: Id, mut data: D) -> Result<D, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::modify(user, |entries: &mut Vec<D>| {
            let index = find(id, entries).ok_or(Error::NotFound(id))?;
            let old = &mut entries[index];
            data.set_id(old.id());
            std::mem::swap(old, &mut data);
            Ok(data)
        })
    }

    fn delete(&mut self, user: &str, id: Id) -> Result<D, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::modify(user, |entries: &mut Vec<D>| {
            let index = find(id, entries).ok_or(Error::NotFound(id))?;
            Ok(entries.remove(index))
        })
    }
}

fn find<D: Data>(id: Id, data: &[D]) -> Option<usize> {
    let index = if data.is_empty() {
        return None;
    } else {
        let index = <usize as std::convert::TryFrom<Id>>::try_from(id).ok()?;
        std::cmp::min(data.len() - 1, index)
    };

    for i in (0..=index).rev() {
        if data[i].id() == id {
            return Some(i);
        }
    }
    None
}

mod fs {
    use super::{Error, InFile, Named};

    #[derive(Debug, Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct User(std::path::PathBuf);

    impl User {
        pub fn new(user: &str, store: &InFile) -> Result<Self, Error> {
            if store.users.contains(user) {
                Ok(Self(store.path.join(user)))
            } else {
                Err(Error::NoSuchUser(String::from(user)))
            }
        }

        pub fn to_path<D: Named>(&self) -> UserPath {
            UserPath(self.0.join(D::name()))
        }
    }

    #[derive(Debug, Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct UserPath(std::path::PathBuf);

    impl UserPath {
        pub fn new<D: Named>(user: &str, store: &InFile) -> Result<Self, Error> {
            if store.users.contains(user) {
                Ok(Self(store.path.join(user).join(D::name())))
            } else {
                Err(Error::NoSuchUser(String::from(user)))
            }
        }
    }

    impl AsRef<std::path::Path> for UserPath {
        fn as_ref(&self) -> &std::path::Path {
            &self.0
        }
    }

    pub fn reader<U: std::borrow::Borrow<UserPath>>(
        user: U,
    ) -> Result<csv::Reader<std::fs::File>, Error> {
        let file = std::fs::File::open(user.borrow())?;
        Ok(csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(file))
    }

    pub fn modify<D, F>(user: UserPath, action: F) -> Result<D, Error>
    where
        D: Named,
        F: FnOnce(&mut Vec<D>) -> Result<D, Error>,
    {
        let mut entries = load(&user)?;
        let data = action(&mut entries)?;
        write(user, entries)?;
        Ok(data)
    }

    fn load<U: std::borrow::Borrow<UserPath>, D: Named>(user: U) -> Result<Vec<D>, Error> {
        let mut reader = reader(user)?;

        let mut entries = vec![];
        for entry in reader.deserialize::<D>() {
            match entry {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    return Err(Error::Serde(e.to_string()));
                }
            }
        }

        Ok(entries)
    }

    fn write<D: Named>(user: UserPath, entries: Vec<D>) -> Result<(), Error> {
        use std::io::Write;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_writer(vec![]);
        for entry in entries {
            writer
                .serialize(entry)
                .map_err(|e| Error::Serde(e.to_string()))?;
        }

        std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(user)?
            .write_all(
                writer
                    .into_inner()
                    .map_err(|e| Error::Serde(e.to_string()))?
                    .as_slice(),
            )
            .map_err(Error::Io)
    }

    pub fn append<D: Named>(user: UserPath, data: D) -> Result<(), Error> {
        let file = std::fs::OpenOptions::new().append(true).open(user)?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_writer(file);
        writer.serialize(data).map_err(|e| Error::Io(e.into()))
    }
}

pub trait Named: Data {
    fn name() -> &'static str;
}

impl Named for Skull {
    fn name() -> &'static str {
        "skull"
    }
}

impl Named for Quick {
    fn name() -> &'static str {
        "quick"
    }
}

impl Named for Occurrence {
    fn name() -> &'static str {
        "occurrence"
    }
}

#[cfg(test)]
mod test {
    use super::{Error, Id, InFile, Skull, Store};

    const USER: &str = "bloink";
    const SKULLS: &str = r#"0	skull	0		0.1	
4	skool	0		0.3	
10	skrut	0		43	
"#;

    struct TestStore {
        store: InFile,
        path: std::path::PathBuf,
    }

    impl TestStore {
        pub fn new() -> Self {
            use rand::Rng;

            let name = format!("{:016x}", rand::thread_rng().gen::<u64>());
            let path = std::env::temp_dir().join("skull-test");
            if !path.exists() {
                std::fs::create_dir(&path).unwrap();
            } else if !path.is_dir() {
                panic!("Cannot use {} as test path", path.display());
            }
            let path = path.join(name);
            if path.exists() {
                panic!(
                    "Cannot use {} as test path as it already exists",
                    path.display()
                );
            }
            std::fs::create_dir(&path).unwrap();
            let store = InFile::new(&path, &[USER]).unwrap_or_else(|e| {
                drop(std::fs::remove_dir_all(&path));
                panic!("{}", e);
            });

            Self { store, path }
        }

        pub fn with_data(self) -> Self {
            std::fs::write(self.path.join(USER).join("skull"), SKULLS).unwrap();
            self
        }

        pub fn verify_skull(&self, payload: &str) {
            let data =
                String::from_utf8(std::fs::read(self.path.join(USER).join("skull")).unwrap())
                    .unwrap();
            assert_eq!(data.as_str(), payload);
        }
    }

    impl std::ops::Deref for TestStore {
        type Target = InFile;

        fn deref(&self) -> &Self::Target {
            &self.store
        }
    }

    impl std::ops::DerefMut for TestStore {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.store
        }
    }

    impl Drop for TestStore {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.path));
        }
    }

    fn new_skull(name: &str, unit_price: f32, id: Id) -> Skull {
        Skull {
            id,
            name: String::from(name),
            color: 0,
            icon: String::new(),
            unit_price,
            limit: None,
        }
    }

    #[test]
    fn reject_unknown_user() {
        let mut store = TestStore::new();
        let skull = new_skull("skull", 0.4, 0);
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
        let mut store = TestStore::new().with_data();

        assert_eq!(
            store.last_modified("unknown").unwrap_err().to_string(),
            Error::NoSuchUser(String::from("unknown")).to_string()
        );

        let mut last_modified = store.last_modified(USER).unwrap();

        store.skull().list(USER).unwrap();
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        store.skull().filter_list(USER, Box::new(|_| true)).unwrap();
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        store
            .skull()
            .create(USER, new_skull("bla", 1.0, 0))
            .unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        store.skull().read(USER, 0).unwrap();
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        store
            .skull()
            .update(USER, 0, new_skull("bla", 2.0, 0))
            .unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        store.skull().delete(USER, 0).unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        assert!(store
            .skull()
            .update(USER, 3, new_skull("bla", 1.0, 0))
            .is_err());
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        assert!(store.skull().delete(USER, 5).is_err());
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        store
            .quick()
            .create(
                USER,
                super::Quick {
                    id: 0,
                    skull: 0,
                    amount: 3.0,
                },
            )
            .unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
    }

    #[test]
    fn create() {
        let mut store = TestStore::new().with_data();
        {
            let skull = new_skull("skull", 0.1, 2);
            let id = store.skull().create(USER, skull).unwrap();
            assert!(id == 11);
        }
        {
            let skull = new_skull("skull", 0.3, 4);
            let id = store.skull().create(USER, skull).unwrap();
            assert!(id == 12);
        }

        store.verify_skull(
            r#"0	skull	0		0.1	
4	skool	0		0.3	
10	skrut	0		43	
11	skull	0		0.1	
12	skull	0		0.3	
"#,
        );
    }

    #[test]
    fn read() {
        let mut store = TestStore::new().with_data();

        let expected = new_skull("skool", 0.3, 4);
        assert_eq!(store.skull().read(USER, 4).unwrap().into_owned(), expected);
        store.verify_skull(SKULLS);
    }

    #[test]
    fn read_not_found() {
        let mut store = TestStore::new().with_data();

        assert_eq!(
            store.skull().read(USER, 1).unwrap_err().to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[test]
    fn update() {
        let mut store = TestStore::new().with_data();

        let old = new_skull("skool", 0.3, 4);
        let new = new_skull("bla", 0.7, 5);

        assert_eq!(store.skull().update(USER, 4, new).unwrap(), old);

        store.verify_skull(
            r#"0	skull	0		0.1	
4	bla	0		0.7	
10	skrut	0		43.0	
"#,
        );
    }

    #[test]
    fn update_not_found() {
        let mut store = TestStore::new().with_data();

        let new = new_skull("bla", 0.7, 5);

        assert_eq!(
            store.skull().update(USER, 1, new).unwrap_err().to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[test]
    fn delete() {
        let mut store = TestStore::new().with_data();

        let old = new_skull("skool", 0.3, 4);

        assert_eq!(store.skull().delete(USER, 4).unwrap(), old);
        store.verify_skull(
            r#"0	skull	0		0.1	
10	skrut	0		43.0	
"#,
        );
    }

    #[test]
    fn delete_not_found() {
        let mut store = TestStore::new().with_data();

        assert_eq!(
            store.skull().delete(USER, 1).unwrap_err().to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn find() {
        let mut store = TestStore::new();
        {
            let mut writer = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_path(store.path.join(USER).join("skull"))
                .unwrap();
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| new_skull("skull", i as f32, i))
                .for_each(|s| writer.serialize(s).unwrap());
        }

        for i in 0..30 {
            assert_eq!(
                store.skull().read(USER, i).is_ok(),
                i % 3 != 0 && i % 4 != 0
            );
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn delete_from_list() {
        let mut store = TestStore::new();

        for i in 0..30 {
            store
                .skull()
                .create(USER, new_skull("skull", i as f32, 0))
                .unwrap();
        }

        for i in 0..30 {
            if i % 3 == 0 || i % 4 == 0 {
                store.skull().delete(USER, i).unwrap();
            }
        }

        let expected = {
            let mut writer = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_writer(vec![]);
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| new_skull("skull", i as f32, i))
                .for_each(|s| writer.serialize(s).unwrap());
            writer.into_inner().unwrap()
        };

        let actual = std::fs::read(store.path.join(USER).join("skull")).unwrap();

        assert_eq!(actual, expected);
    }
}
