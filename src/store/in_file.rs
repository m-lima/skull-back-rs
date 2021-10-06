use super::{Crud, Data, Error, Id, LastModified, Occurrence, Quick, Skull, Store, WithId};

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

impl<D: Fileable> Crud<D> for InFile {
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .map(|reader| {
                reader
                    .filter_map(Result::ok)
                    .map(D::read)
                    .filter_map(Result::ok)
                    .map(std::borrow::Cow::Owned)
                    .collect()
            })
    }

    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&WithId<D>) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, WithId<D>>>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .map(|reader| {
                reader
                    .filter_map(Result::ok)
                    .map(D::read)
                    .filter_map(Result::ok)
                    .filter(|d| (filter)(d))
                    .map(std::borrow::Cow::Owned)
                    .collect()
            })
    }

    fn create(&mut self, user: &str, data: D) -> Result<Id, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::reader(&user)
            .map(|reader| {
                reader
                    .filter_map(Result::ok)
                    .map(D::read)
                    .filter_map(Result::ok)
                    .last()
                    .map_or(0, |d| d.id + 1)
            })
            .and_then(|id| {
                let with_id = WithId::new(id, data);
                fs::append(user, &with_id)?;
                Ok(id)
            })
    }

    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, WithId<D>>, Error> {
        fs::UserPath::new::<D>(user, self)
            .and_then(fs::reader)
            .and_then(|reader| {
                reader
                    .filter_map(Result::ok)
                    .map(D::read)
                    .filter_map(Result::ok)
                    .find(|d| d.id == id)
                    .map(std::borrow::Cow::Owned)
                    .ok_or(Error::NotFound(id))
            })
    }

    fn update(&mut self, user: &str, id: Id, data: D) -> Result<WithId<D>, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::modify(user, |entries| {
            let index = find(id, entries).ok_or(Error::NotFound(id))?;
            let old = &mut entries[index];
            let mut with_id = WithId::new(old.id, data);
            std::mem::swap(old, &mut with_id);
            Ok(with_id)
        })
    }

    fn delete(&mut self, user: &str, id: Id) -> Result<WithId<D>, Error> {
        let user = fs::UserPath::new::<D>(user, self)?;

        fs::modify(user, |entries| {
            let index = find(id, entries).ok_or(Error::NotFound(id))?;
            Ok(entries.remove(index))
        })
    }
}

fn find<D: Data>(id: Id, data: &[WithId<D>]) -> Option<usize> {
    let index = if data.is_empty() {
        return None;
    } else {
        let index = <usize as std::convert::TryFrom<Id>>::try_from(id).ok()?;
        std::cmp::min(data.len() - 1, index)
    };

    for i in (0..=index).rev() {
        if data[i].id == id {
            return Some(i);
        }
    }
    None
}

mod fs {
    use super::{Error, Fileable, InFile, WithId};

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

        pub fn to_path<D: Fileable>(&self) -> UserPath {
            UserPath(self.0.join(D::name()))
        }
    }

    #[derive(Debug, Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct UserPath(std::path::PathBuf);

    impl UserPath {
        pub fn new<D: Fileable>(user: &str, store: &InFile) -> Result<Self, Error> {
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
    ) -> Result<std::io::Lines<std::io::BufReader<std::fs::File>>, Error> {
        use std::io::BufRead;
        Ok(std::io::BufReader::new(std::fs::File::open(user.borrow())?).lines())
    }

    pub fn modify<D, F>(user: UserPath, action: F) -> Result<WithId<D>, Error>
    where
        D: Fileable,
        F: FnOnce(&mut Vec<WithId<D>>) -> Result<WithId<D>, Error>,
    {
        let mut entries = load(&user)?;
        let data = action(&mut entries)?;
        write(user, entries)?;
        Ok(data)
    }

    fn load<U: std::borrow::Borrow<UserPath>, D: Fileable>(
        user: U,
    ) -> Result<Vec<WithId<D>>, Error> {
        let mut entries = vec![];
        for entry in reader(user)?.map(|e| e.map_err(Error::Io).and_then(D::read)) {
            match entry {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    return Err(Error::Serde(e.to_string()));
                }
            }
        }

        Ok(entries)
    }

    fn write<D: Fileable>(user: UserPath, entries: Vec<WithId<D>>) -> Result<(), Error> {
        use std::io::Write;

        let mut buffer = vec![];
        for entry in entries {
            D::write(&entry, &mut buffer)?;
        }

        std::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(user)?
            .write_all(buffer.as_slice())
            .map_err(Error::Io)
    }

    pub fn append<D: Fileable>(user: UserPath, data: &WithId<D>) -> Result<(), Error> {
        let mut file = std::fs::OpenOptions::new().append(true).open(user)?;
        D::write(data, &mut file)
    }
}

pub trait Fileable: Data {
    fn name() -> &'static str;
    fn read(string: String) -> Result<WithId<Self>, Error>;
    fn write<W: std::io::Write>(with_id: &WithId<Self>, writer: &mut W) -> Result<(), Error>;
}

impl Fileable for Skull {
    fn name() -> &'static str {
        "skull"
    }

    fn read(string: String) -> Result<WithId<Self>, Error> {
        let mut split = string.split('\t');

        let id = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `id` found for Skull")))
            .and_then(|v| {
                v.parse()
                    .map_err(|e| Error::Serde(format!("Could not parse `id` for Skull: {}", e)))
            })?;

        let name = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `name` found for Skull")))
            .map(String::from)?;

        let color = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `color` found for Skull")))
            .map(String::from)?;

        let icon = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `icon` found for Skull")))
            .map(String::from)?;

        let unit_price = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `unit_price` found for Skull")))
            .and_then(|v| {
                v.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `unit_price` for Skull: {}", e))
                })
            })?;

        let limit =
            if let Some(limit) = split.next().filter(|v| !v.is_empty()) {
                if split.next().is_some() {
                    return Err(Error::Serde(String::from("Too many fields for Skull")));
                }
                Some(limit.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `limit` for Skull: {}", e))
                })?)
            } else {
                None
            };

        Ok(WithId::new(
            id,
            Self {
                name,
                color,
                icon,
                unit_price,
                limit,
            },
        ))
    }

    fn write<W: std::io::Write>(with_id: &WithId<Self>, writer: &mut W) -> Result<(), Error> {
        let data = &with_id.data;

        write!(
            writer,
            "{}\t{}\t{}\t{}\t{}\t",
            with_id.id, data.name, data.color, data.icon, data.unit_price,
        )
        .map_err(Error::Io)?;

        if let Some(limit) = data.limit {
            writeln!(writer, "{}", limit)
        } else {
            writeln!(writer,)
        }
        .map_err(Error::Io)
    }
}

impl Fileable for Quick {
    fn name() -> &'static str {
        "quick"
    }

    fn read(string: String) -> Result<WithId<Self>, Error> {
        let mut split = string.split('\t');

        let id = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `id` found for Quick")))
            .and_then(|v| {
                v.parse()
                    .map_err(|e| Error::Serde(format!("Could not parse `id` for Quick: {}", e)))
            })?;

        let skull = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `skull` found for Quick")))
            .and_then(|v| {
                v.parse()
                    .map_err(|e| Error::Serde(format!("Could not parse `skull` for Quick: {}", e)))
            })?;

        let amount = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `amount` found for Quick")))
            .and_then(|v| {
                v.parse()
                    .map_err(|e| Error::Serde(format!("Could not parse `amount` for Quick: {}", e)))
            })?;

        if split.next().is_some() {
            return Err(Error::Serde(String::from("Too many fields for Quick")));
        }

        Ok(WithId::new(id, Self { skull, amount }))
    }

    fn write<W: std::io::Write>(with_id: &WithId<Self>, writer: &mut W) -> Result<(), Error> {
        let data = &with_id.data;
        writeln!(writer, "{}\t{}\t{}", with_id.id, data.skull, data.amount).map_err(Error::Io)
    }
}

impl Fileable for Occurrence {
    fn name() -> &'static str {
        "occurrence"
    }

    fn read(string: String) -> Result<WithId<Self>, Error> {
        let mut split = string.split('\t');

        let id = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `id` found for Occurrence")))
            .and_then(|v| {
                v.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `id` for Occurrence: {}", e))
                })
            })?;

        let skull = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `skull` found for Occurrence")))
            .and_then(|v| {
                v.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `skull` for Occurrence: {}", e))
                })
            })?;

        let amount = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `amount` found for Occurrence")))
            .and_then(|v| {
                v.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `amount` for Occurrence: {}", e))
                })
            })?;

        let timestamp = split
            .next()
            .ok_or_else(|| Error::Serde(String::from("No `timestamp` found for Occurrence")))
            .and_then(|v| {
                v.parse().map_err(|e| {
                    Error::Serde(format!("Could not parse `amount` for Occurrence: {}", e))
                })
            })
            .and_then(|v| {
                std::time::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(v))
                    .ok_or(Error::BadTimestamp)
            })?;

        if split.next().is_some() {
            return Err(Error::Serde(String::from("Too many fields for Occurrence")));
        }

        Ok(WithId::new(
            id,
            Self {
                skull,
                amount,
                timestamp,
            },
        ))
    }

    fn write<W: std::io::Write>(with_id: &WithId<Self>, writer: &mut W) -> Result<(), Error> {
        let data = &with_id.data;
        writeln!(
            writer,
            "{}\t{}\t{}\t{}",
            with_id.id,
            data.skull,
            data.amount,
            data.timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| Error::BadTimestamp)?
                .as_millis()
        )
        .map_err(Error::Io)
    }
}

#[cfg(test)]
mod test {
    use super::{Error, Fileable, InFile, Skull, Store, WithId};

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

    fn new_skull(name: &str, unit_price: f32) -> Skull {
        Skull {
            name: String::from(name),
            color: String::from('0'),
            icon: String::new(),
            unit_price,
            limit: None,
        }
    }

    #[test]
    fn reject_unknown_user() {
        let mut store = TestStore::new();
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

        store.skull().create(USER, new_skull("bla", 1.0)).unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        store.skull().read(USER, 0).unwrap();
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        store
            .skull()
            .update(USER, 0, new_skull("bla", 2.0))
            .unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        store.skull().delete(USER, 0).unwrap();
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
        last_modified = store.last_modified(USER).unwrap();

        assert!(store
            .skull()
            .update(USER, 3, new_skull("bla", 1.0))
            .is_err());
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

        assert!(store.skull().delete(USER, 5).is_err());
        assert_eq!(store.last_modified(USER).unwrap(), last_modified);

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
        assert_ne!(store.last_modified(USER).unwrap(), last_modified);
    }

    #[test]
    fn create() {
        let mut store = TestStore::new().with_data();
        {
            let skull = new_skull("skull", 0.1);
            let id = store.skull().create(USER, skull).unwrap();
            assert!(id == 11);
        }
        {
            let skull = new_skull("skull", 0.3);
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

        let expected = WithId::new(4, new_skull("skool", 0.3));
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

        let old = WithId::new(4, new_skull("skool", 0.3));
        let new = new_skull("bla", 0.7);

        assert_eq!(store.skull().update(USER, 4, new).unwrap(), old);

        store.verify_skull(
            r#"0	skull	0		0.1	
4	bla	0		0.7	
10	skrut	0		43	
"#,
        );
    }

    #[test]
    fn update_not_found() {
        let mut store = TestStore::new().with_data();

        let new = new_skull("bla", 0.7);

        assert_eq!(
            store.skull().update(USER, 1, new).unwrap_err().to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[test]
    fn delete() {
        let mut store = TestStore::new().with_data();

        let old = WithId::new(4, new_skull("skool", 0.3));

        assert_eq!(store.skull().delete(USER, 4).unwrap(), old);
        store.verify_skull(
            r#"0	skull	0		0.1	
10	skrut	0		43	
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
            let mut file = std::fs::File::create(store.path.join(USER).join("skull")).unwrap();
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| WithId::new(i, new_skull("skull", i as f32)))
                .for_each(|s| Fileable::write(&s, &mut file).unwrap());
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
                .create(USER, new_skull("skull", i as f32))
                .unwrap();
        }

        for i in 0..30 {
            if i % 3 == 0 || i % 4 == 0 {
                store.skull().delete(USER, i).unwrap();
            }
        }

        let expected = {
            let mut expected = vec![];
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| WithId::new(i, new_skull("skull", i as f32)))
                .for_each(|s| Fileable::write(&s, &mut expected).unwrap());
            expected
        };

        let actual = std::fs::read(store.path.join(USER).join("skull")).unwrap();

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod bench {

    mod handwritten {
        extern crate test;
        use super::super::{Fileable, Skull, WithId};

        #[bench]
        fn serialize(bench: &mut test::Bencher) {
            let skull = Skull {
                name: String::from("xnamex"),
                color: String::from("xcolorx"),
                icon: String::from("xiconx"),
                unit_price: 0.1,
                limit: None,
            };

            bench.iter(|| {
                let mut buffer = vec![];

                (0..100)
                    .map(|i| WithId::new(i, skull.clone()))
                    .for_each(|s| Fileable::write(&s, &mut buffer).unwrap());
            });
        }

        #[bench]
        fn deserialize(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|i| format!("{}\txnamex\txcolorx\txiconx\t1.2\t{}", i, i))
                .collect::<Vec<_>>();

            bench.iter(|| {
                let data = data.clone();

                for (i, string) in data.into_iter().enumerate() {
                    let s = <Skull as Fileable>::read(string).unwrap();
                    assert_eq!(s.id, i as u32);
                    let s = s.data;
                    assert_eq!(s.name, "xnamex");
                    assert_eq!(s.color, "xcolorx");
                    assert_eq!(s.icon, "xiconx");
                    assert_eq!(s.unit_price, 1.2);
                    assert_eq!(s.limit.unwrap(), i as f32);
                }
            });
        }
    }

    mod csv {
        extern crate test;
        use super::super::Skull;

        #[bench]
        fn serialize(bench: &mut test::Bencher) {
            let skull = Skull {
                name: String::from("xnamex"),
                color: String::from("xcolorx"),
                icon: String::from("xiconx"),
                unit_price: 0.1,
                limit: None,
            };

            bench.iter(|| {
                let buffer = vec![];

                let mut writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_writer(buffer);

                (0..100)
                    .map(|i| {
                        let mut s = skull.clone();
                        s.unit_price = i as f32;
                        s
                    })
                    .for_each(|s| writer.serialize(s).unwrap());
            });
        }

        #[bench]
        fn deserialize(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|i| format!("xnamex\txcolorx\txiconx\t1.2\t{}\n", i))
                .map(|s| s.into_bytes())
                .flatten()
                .collect::<Vec<_>>();

            bench.iter(|| {
                let data = data.clone();

                let mut reader = csv::ReaderBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_reader(data.as_slice());

                reader
                    .deserialize::<Skull>()
                    .enumerate()
                    .for_each(|(i, s)| {
                        let s = s.unwrap();
                        assert_eq!(s.name, "xnamex");
                        assert_eq!(s.color, "xcolorx");
                        assert_eq!(s.icon, "xiconx");
                        assert_eq!(s.unit_price, 1.2);
                        assert_eq!(s.limit.unwrap(), i as f32);
                    })
            });
        }
    }
}
