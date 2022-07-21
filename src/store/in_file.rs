use super::{Crud, Data, Error, Id, Occurrence, Quick, Skull, Store, WithId};

#[cfg(all(test, nightly))]
mod serde;

macro_rules! parse {
    (string, $input:expr, $field:literal, $data:literal) => {
        $input
            .next()
            .ok_or_else(|| parse!(not_found, $field, $data))
            .map(String::from)
    };

    (number, $input:expr, $field:literal, $data:literal) => {
        $input
            .next()
            .ok_or_else(|| parse!(not_found, $field, $data))
            .and_then(|v| parse!(number_raw, v, $field, $data))
    };

    (end, $input:expr, $data:literal) => {
        if $input.next().is_some() {
            return Err(Error::Serde(String::from(concat!(
                "Too many fields for ",
                $data
            ))));
        }
    };

    (number_raw, $input:expr, $field:literal, $data:literal) => {
        $input.parse().map_err(|e| {
            Error::Serde(format!(
                concat!("Could not parse `", $field, "` for ", $data, ": {}"),
                e
            ))
        })
    };

    (not_found, $field:literal, $data:literal) => {
        Error::Serde(String::from(concat!(
            "Could not find `",
            $field,
            "` for ",
            $data
        )))
    };
}

macro_rules! write_number {
    ($serializer:ident, $writer:expr, $value:expr, $field:literal, $data:literal) => {{
        $writer
            .write_all($serializer::Buffer::new().format($value).as_bytes())
            .map_err(|e| {
                Error::Serde(format!(
                    concat!("Could not serialize `", $field, "` for ", $data, ": {}"),
                    e
                ))
            })
    }};
}

pub struct InFile {
    users: std::collections::HashMap<String, UserStore>,
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
            .map_err(|e| anyhow::anyhow!("Store directory cannot be read: {e}"))?;

        let users = users
            .into_iter()
            .map(|user| path.join(user.as_ref()))
            .chain(
                dir_reader
                    .filter_map(Result::ok)
                    .map(|dir| dir.path())
                    .filter(|dir| dir.is_dir()),
            )
            .filter_map(|root| {
                root.file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(String::from)
                    .map(|name| (name, root))
            })
            .collect::<std::collections::HashSet<_>>();

        for (user, path) in &users {
            if !path.exists() {
                log::debug!("Creating {}", path.display());
                std::fs::create_dir(&path).map_err(|e| {
                    anyhow::anyhow!("Could not create user directory {}: {e}", path.display())
                })?;
            } else if !path.is_dir() {
                anyhow::bail!("User path is not a directory {}", path.display());
            }

            for file in
                [Skull::name(), Quick::name(), Occurrence::name()].map(|name| path.join(name))
            {
                if !file.exists() {
                    log::debug!("Creating {}", path.display());
                    std::fs::File::create(&file)
                        .map_err(|e| anyhow::anyhow!("Could not create {}: {e}", file.display()))?;
                } else if file.is_dir() {
                    anyhow::bail!("Path {} is not a file", file.display());
                }
            }
            log::info!("Allowing {user}");
        }

        let users = users
            .into_iter()
            .map(|(user, path)| {
                let skull = std::sync::RwLock::new(UserFile::new(path.join(Skull::name())));
                let quick = std::sync::RwLock::new(UserFile::new(path.join(Quick::name())));
                let occurrence =
                    std::sync::RwLock::new(UserFile::new(path.join(Occurrence::name())));
                (
                    user,
                    UserStore {
                        skull,
                        quick,
                        occurrence,
                    },
                )
            })
            .collect();

        Ok(Self { users })
    }
}

impl Store for InFile {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error> {
        let user = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(&user.skull)
    }

    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error> {
        let user = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(&user.quick)
    }

    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error> {
        let user = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(&user.occurrence)
    }
}

struct UserStore {
    skull: std::sync::RwLock<UserFile<Skull>>,
    quick: std::sync::RwLock<UserFile<Quick>>,
    occurrence: std::sync::RwLock<UserFile<Occurrence>>,
}

struct UserFile<D: Data> {
    file: std::path::PathBuf,
    _marker: std::marker::PhantomData<D>,
}

impl<D: FileData> UserFile<D> {
    fn new(file: std::path::PathBuf) -> Self {
        Self {
            file,
            _marker: std::marker::PhantomData,
        }
    }

    fn lines(&self) -> Result<impl Iterator<Item = std::io::Result<String>>, Error> {
        use std::io::BufRead;
        let iter = std::io::BufReader::new(std::fs::File::open(&self.file)?).lines();
        Ok(iter)
    }

    fn replace(&mut self, entries: Vec<D::Id>) -> Result<(), Error> {
        use std::io::Write;

        let mut buffer = vec![];
        for entry in entries {
            D::write_tsv(entry, &mut buffer)?;
        }

        std::fs::File::options()
            .truncate(true)
            .write(true)
            .open(&self.file)?
            .write_all(buffer.as_slice())
            .map_err(Error::Io)
    }

    fn good_line<T>(&self, line: (usize, Result<T, Error>)) -> Option<T> {
        match line.1 {
            Ok(line) => Some(line),
            Err(err) => {
                log::error!("Failed to read {}:{}: {err}", self.file.display(), line.0);
                None
            }
        }
    }

    fn good_line_with_index(
        &self,
        line: (usize, Result<D::Id, Error>),
        id: Id,
        index: &mut Option<usize>,
    ) -> Option<D::Id> {
        match line.1 {
            Ok(entry) => {
                if entry.id() == id {
                    *index = Some(line.0);
                }
                Some(entry)
            }
            Err(err) => {
                log::error!("Failed to read {}:{}: {err}", self.file.display(), line.0);
                None
            }
        }
    }
}

#[async_trait::async_trait]
impl<D: FileData> Crud<D> for std::sync::RwLock<UserFile<D>> {
    async fn list(&self, limit: Option<usize>) -> Result<Vec<D::Id>, Error> {
        let lock = self.read()?;
        let entries = lock
            .lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .collect::<Vec<_>>();
        if let Some(limit) = limit {
            let len = entries.len();
            Ok(entries.into_iter().skip(len - limit).collect())
        } else {
            Ok(entries)
        }
    }

    async fn create(&self, data: D) -> Result<Id, Error> {
        let lock = self.write()?;
        let id = lock
            .lines()?
            .map(D::id)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .max()
            .map_or(0, |id| id + 1);

        let mut file = std::fs::File::options().append(true).open(&lock.file)?;
        D::write_tsv(D::Id::new(id, data), &mut file)?;

        Ok(id)
    }

    async fn read(&self, id: Id) -> Result<D::Id, Error> {
        let lock = self.read()?;
        lock.lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .find(|d| d.id() == id)
            .ok_or(Error::NotFound(id))
    }

    async fn update(&self, id: Id, data: D) -> Result<D::Id, Error> {
        let mut lock = self.write()?;
        let mut index = None;
        let mut entries = lock
            .lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line_with_index(line, id, &mut index))
            .collect::<Vec<_>>();

        let index = index.ok_or(Error::NotFound(id))?;

        let old = &mut entries[index];
        let mut new = D::Id::new(id, data);
        std::mem::swap(old, &mut new);

        lock.replace(entries)?;
        Ok(new)
    }

    async fn delete(&self, id: Id) -> Result<D::Id, Error> {
        let mut lock = self.write()?;
        let mut index = None;
        let mut entries = lock
            .lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line_with_index(line, id, &mut index))
            .collect::<Vec<_>>();

        let index = index.ok_or(Error::NotFound(id))?;
        let old = entries.remove(index);

        lock.replace(entries)?;
        Ok(old)
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let lock = self.read()?;
        std::fs::metadata(&lock.file)
            .and_then(|f| f.modified())
            .map_err(Error::Io)
    }
}

pub trait FileData: super::Data {
    fn name() -> &'static str;
    fn id(string: std::io::Result<String>) -> Result<Id, Error>;
    fn read_tsv(string: std::io::Result<String>) -> Result<Self::Id, Error>;
    fn write_tsv<W: std::io::Write>(with_id: Self::Id, writer: &mut W) -> Result<(), Error>;
}

impl FileData for Skull {
    fn name() -> &'static str {
        "skull"
    }

    fn id(string: std::io::Result<String>) -> Result<Id, Error> {
        let string = string?;
        parse!(number, string.split('\t'), "id", "Skull")
    }

    fn read_tsv(string: std::io::Result<String>) -> Result<Self::Id, Error> {
        let string = string?;
        let mut split = string.split('\t');

        let id = parse!(number, split, "id", "Skull")?;
        let name = parse!(string, split, "name", "Skull")?;
        let color = parse!(string, split, "color", "Skull")?;
        let icon = parse!(string, split, "icon", "Skull")?;
        let unit_price = parse!(number, split, "unit_price", "Skull")?;

        let limit = if let Some(limit) = split.next().filter(|v| !v.is_empty()) {
            Some(parse!(number_raw, limit, "limit", "Skull")?)
        } else {
            None
        };
        parse!(end, split, "Skull");

        Ok(Self::Id::new(
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

    fn write_tsv<W: std::io::Write>(data: Self::Id, writer: &mut W) -> Result<(), Error> {
        write_number!(itoa, writer, data.id(), "id", "Skull")?;

        writer.write_all(b"\t")?;
        writer.write_all(data.name.as_bytes())?;
        writer.write_all(b"\t")?;
        writer.write_all(data.color.as_bytes())?;
        writer.write_all(b"\t")?;
        writer.write_all(data.icon.as_bytes())?;
        writer.write_all(b"\t")?;

        write_number!(ryu, writer, data.unit_price, "unit_price", "Skull")?;
        writer.write_all(b"\t")?;

        if let Some(limit) = data.limit {
            write_number!(ryu, writer, limit, "limit", "Skull")?;
        }

        writer.write_all(b"\n").map_err(Error::Io)
    }
}

impl FileData for Quick {
    fn name() -> &'static str {
        "quick"
    }

    fn id(string: std::io::Result<String>) -> Result<Id, Error> {
        let string = string?;
        parse!(number, string.split('\t'), "id", "Quick")
    }

    fn read_tsv(string: std::io::Result<String>) -> Result<Self::Id, Error> {
        let string = string?;
        let mut split = string.split('\t');

        let id = parse!(number, split, "id", "Quick")?;
        let skull = parse!(number, split, "skull", "Quick")?;
        let amount = parse!(number, split, "amount", "Quick")?;
        parse!(end, split, "Quick");

        Ok(Self::Id::new(id, Self { skull, amount }))
    }

    fn write_tsv<W: std::io::Write>(data: Self::Id, writer: &mut W) -> Result<(), Error> {
        write_number!(itoa, writer, data.id(), "id", "Quick")?;
        writer.write_all(b"\t")?;
        write_number!(itoa, writer, data.skull, "skull", "Quick")?;
        writer.write_all(b"\t")?;
        write_number!(ryu, writer, data.amount, "amount", "Quick")?;
        writer.write_all(b"\n").map_err(Error::Io)
    }
}

impl FileData for Occurrence {
    fn name() -> &'static str {
        "occurrence"
    }

    fn id(string: std::io::Result<String>) -> Result<Id, Error> {
        let string = string?;
        parse!(number, string.split('\t'), "id", "Occurrence")
    }

    fn read_tsv(string: std::io::Result<String>) -> Result<Self::Id, Error> {
        let string = string?;
        let mut split = string.split('\t');

        let id = parse!(number, split, "id", "Occurrence")?;
        let skull = parse!(number, split, "skull", "Occurrence")?;
        let amount = parse!(number, split, "amount", "Occurrence")?;
        let millis = parse!(number, split, "millis", "Occurrence")?;
        parse!(end, split, "Occurrence");

        Ok(Self::Id::new(
            id,
            Self {
                skull,
                amount,
                millis,
            },
        ))
    }

    // Allowed because u64 millis is already many times the age of the universe
    #[allow(clippy::cast_possible_truncation)]
    fn write_tsv<W: std::io::Write>(data: Self::Id, writer: &mut W) -> Result<(), Error> {
        write_number!(itoa, writer, data.id(), "id", "Occurrence")?;
        writer.write_all(b"\t")?;
        write_number!(itoa, writer, data.skull, "skull", "Occurrence")?;
        writer.write_all(b"\t")?;
        write_number!(ryu, writer, data.amount, "amount", "Occurrence")?;
        writer.write_all(b"\t")?;
        write_number!(itoa, writer, data.millis, "millis", "Occurrence")?;
        writer.write_all(b"\n").map_err(Error::Io)
    }
}

#[cfg(test)]
mod test {
    use crate::store::{Quick, Selector};

    use super::{Error, FileData, InFile, Skull, Store, WithId};

    type SkullId = <Skull as super::Data>::Id;

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
            if path.exists() {
                assert!(path.is_dir(), "Cannot use {} as test path", path.display());
            } else {
                std::fs::create_dir(&path).unwrap();
            }
            let path = path.join(name);
            assert!(
                !path.exists(),
                "Cannot use {} as test path as it already exists",
                path.display()
            );
            std::fs::create_dir(&path).unwrap();
            let store = InFile::new(&path, &[USER]).unwrap_or_else(|e| {
                drop(std::fs::remove_dir_all(&path));
                panic!("{e}");
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

    impl Store for TestStore {
        fn skull(&self, user: &str) -> Result<&dyn crate::store::Crud<Skull>, Error> {
            self.store.skull(user)
        }

        fn quick(&self, user: &str) -> Result<&dyn crate::store::Crud<crate::store::Quick>, Error> {
            self.store.quick(user)
        }

        fn occurrence(
            &self,
            user: &str,
        ) -> Result<&dyn crate::store::Crud<crate::store::Occurrence>, Error> {
            self.store.occurrence(user)
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
        let store = TestStore::new();
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
        let store = TestStore::new().with_data();

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
        let store = TestStore::new().with_data();
        let skulls = Skull::select(&store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .len();
        assert_eq!(skulls, 3);

        let skulls = Skull::select(&store, USER)
            .unwrap()
            .list(Some(1))
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(skulls, vec![SkullId::new(10, new_skull("skrut", 43.0))]);

        let skulls = Skull::select(&store, USER)
            .unwrap()
            .list(Some(0))
            .await
            .unwrap()
            .len();
        assert_eq!(skulls, 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create() {
        let store = TestStore::new().with_data();
        {
            let skull = new_skull("skull", 0.1);
            let id = Skull::select(&store, USER)
                .unwrap()
                .create(skull)
                .await
                .unwrap();
            assert!(id == 11);
        }
        {
            let skull = new_skull("skull", 0.3);
            let id = Skull::select(&store, USER)
                .unwrap()
                .create(skull)
                .await
                .unwrap();
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

    #[tokio::test(flavor = "multi_thread")]
    async fn read() {
        let store = TestStore::new().with_data();

        let expected = SkullId::new(4, new_skull("skool", 0.3));
        let read = Skull::select(&store, USER).unwrap().read(4).await.unwrap();
        assert_eq!(read, expected);
        store.verify_skull(SKULLS);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn read_not_found() {
        let store = TestStore::new().with_data();

        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .read(1)
                .await
                .unwrap_err()
                .to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let store = TestStore::new().with_data();

        let old = SkullId::new(4, new_skull("skool", 0.3));
        let new = new_skull("bla", 0.7);

        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .update(4, new)
                .await
                .unwrap(),
            old
        );

        store.verify_skull(
            r#"0	skull	0		0.1	
4	bla	0		0.7	
10	skrut	0		43.0	
"#,
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn update_not_found() {
        let store = TestStore::new().with_data();

        let new = new_skull("bla", 0.7);

        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .update(1, new)
                .await
                .unwrap_err()
                .to_string(),
            Error::NotFound(1).to_string()
        );
        store.verify_skull(SKULLS);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete() {
        let store = TestStore::new().with_data();

        let old = SkullId::new(4, new_skull("skool", 0.3));

        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .delete(4)
                .await
                .unwrap(),
            old
        );

        store.verify_skull(
            r#"0	skull	0		0.1	
10	skrut	0		43.0	
"#,
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_not_found() {
        let store = TestStore::new().with_data();

        assert_eq!(
            Skull::select(&store, USER)
                .unwrap()
                .delete(1)
                .await
                .unwrap_err()
                .to_string(),
            Error::NotFound(1).to_string()
        );

        store.verify_skull(SKULLS);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(clippy::cast_precision_loss)]
    async fn find() {
        let store = TestStore::new();
        {
            let mut file = std::fs::File::create(store.path.join(USER).join("skull")).unwrap();
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| SkullId::new(i, new_skull("skull", i as f32)))
                .for_each(|s| Skull::write_tsv(s, &mut file).unwrap());
        }

        for i in 0..30 {
            assert_eq!(
                Skull::select(&store, USER).unwrap().read(i).await.is_ok(),
                i % 3 != 0 && i % 4 != 0
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(clippy::cast_precision_loss)]
    async fn delete_from_list() {
        let store = TestStore::new();

        for i in 0..30 {
            Skull::select(&store, USER)
                .unwrap()
                .create(new_skull("skull", i as f32))
                .await
                .unwrap();
        }

        for i in 0..30 {
            if i % 3 == 0 || i % 4 == 0 {
                Skull::select(&store, USER)
                    .unwrap()
                    .delete(i)
                    .await
                    .unwrap();
            }
        }

        let expected = {
            let mut expected = vec![];
            (0..30)
                .filter(|i| i % 3 != 0 && i % 4 != 0)
                .map(|i| SkullId::new(i, new_skull("skull", i as f32)))
                .for_each(|s| Skull::write_tsv(s, &mut expected).unwrap());
            expected
        };

        let actual = std::fs::read(store.path.join(USER).join("skull")).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn error_message() {
        struct FailedWriter;
        impl std::io::Write for FailedWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    Error::Serde(String::from("write")),
                ))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    Error::Serde(String::from("flush")),
                ))
            }
        }

        let field_not_present = "2";
        assert_eq!(
            Skull::read_tsv(Ok(String::from(field_not_present)))
                .unwrap_err()
                .to_string(),
            String::from("Serde error: Could not find `name` for Skull")
        );

        let field_unparseable = "a";
        assert_eq!(
            Skull::read_tsv(Ok(String::from(field_unparseable)))
                .unwrap_err()
                .to_string(),
            String::from(
                "Serde error: Could not parse `id` for Skull: invalid digit found in string"
            )
        );

        let too_many_fields = "2\t\t\t\t2\t\t";
        assert_eq!(
            Skull::read_tsv(Ok(String::from(too_many_fields)))
                .unwrap_err()
                .to_string(),
            String::from("Serde error: Too many fields for Skull")
        );

        let skull = SkullId::new(0, new_skull("skull", 0.0));
        let mut writer = FailedWriter;
        assert_eq!(
            Skull::write_tsv(skull, &mut writer)
                .unwrap_err()
                .to_string(),
            String::from("Serde error: Could not serialize `id` for Skull: Serde error: write")
        );
    }
}

#[cfg(all(test, nightly))]
mod bench {

    mod handwritten {
        extern crate test;
        use super::super::{FileData, Occurrence, Skull, WithId};

        type SkullId = <Skull as super::super::Data>::Id;
        type OccurrenceId = <Occurrence as super::super::Data>::Id;

        #[bench]
        fn serialize_skull(bench: &mut test::Bencher) {
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
                    .map(|i| SkullId::new(i, skull.clone()))
                    .for_each(|s| Skull::write_tsv(s, &mut buffer).unwrap());
            });
        }

        #[bench]
        fn deserialize_skull(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|i| format!("{i}\txnamex\txcolorx\txiconx\t1.2\t{i}"))
                .collect::<Vec<_>>();

            bench.iter(|| {
                let data = data.clone();

                for (i, string) in data.into_iter().enumerate() {
                    let s = Skull::read_tsv(Ok(string)).unwrap();
                    assert_eq!(s.id, i as u32);
                    assert_eq!(s.name, "xnamex");
                    assert_eq!(s.color, "xcolorx");
                    assert_eq!(s.icon, "xiconx");
                    assert_eq!(s.unit_price, 1.2);
                    assert_eq!(s.limit.unwrap(), i as f32);
                }
            });
        }

        #[bench]
        fn serialize_occurrence(bench: &mut test::Bencher) {
            let occurrence = Occurrence {
                skull: 0,
                amount: 1.2,
                millis: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            };

            bench.iter(|| {
                let mut buffer = vec![];

                (0..100)
                    .map(|i| OccurrenceId::new(i, occurrence.clone()))
                    .for_each(|s| Occurrence::write_tsv(s, &mut buffer).unwrap());
            });
        }

        #[bench]
        fn deserialize_occurrence(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|i| format!("{i}\t0\t1.2\t4"))
                .collect::<Vec<_>>();

            bench.iter(|| {
                let data = data.clone();

                for (i, string) in data.into_iter().enumerate() {
                    let s = Occurrence::read_tsv(Ok(string)).unwrap();
                    assert_eq!(s.id, i as u32);
                    assert_eq!(s.skull, 0);
                    assert_eq!(s.amount, 1.2);
                    assert_eq!(s.millis, 4);
                }
            });
        }
    }

    mod serde {
        extern crate test;
        use super::super::{serde::Serde, Occurrence, Skull, WithId};

        type SkullId = <Skull as super::super::Data>::Id;
        type OccurrenceId = <Occurrence as super::super::Data>::Id;

        #[bench]
        fn serialize_skull(bench: &mut test::Bencher) {
            let skull = Skull {
                name: String::from("xnamex"),
                color: String::from("xcolorx"),
                icon: String::from("xiconx"),
                unit_price: 0.1,
                limit: None,
            };

            bench.iter(|| {
                let mut buffer = vec![];
                let mut serder = Serde::new(&mut buffer);

                (0..100)
                    .map(|i| SkullId::new(i, skull.clone()))
                    .for_each(|s| {
                        serde::Serialize::serialize(&s, &mut serder).unwrap();
                    });
            });
        }

        #[bench]
        fn serialize_occurrence(bench: &mut test::Bencher) {
            let occurrence = Occurrence {
                skull: 0,
                amount: 1.2,
                millis: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            };

            bench.iter(|| {
                let mut buffer = vec![];
                let mut serder = super::super::serde::Serde::new(&mut buffer);

                (0..100)
                    .map(|i| OccurrenceId::new(i, occurrence.clone()))
                    .for_each(|s| {
                        serde::Serialize::serialize(&s, &mut serder).unwrap();
                    });
            });
        }
    }

    mod csv {
        extern crate test;
        use super::super::{Occurrence, Skull, WithId};

        type SkullId = <Skull as super::super::Data>::Id;
        type OccurrenceId = <Occurrence as super::super::Data>::Id;

        #[bench]
        fn serialize_skull(bench: &mut test::Bencher) {
            let skull = SkullId {
                id: 0,
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
                        s.id = i;
                        s
                    })
                    .for_each(|s| writer.serialize(s).unwrap());
            });
        }

        #[bench]
        fn deserialize_skull(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|i| format!("xnamex\txcolorx\txiconx\t1.2\t{i}\n"))
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

        #[bench]
        fn serialize_occurrence(bench: &mut test::Bencher) {
            let occurrence = Occurrence {
                skull: 0,
                amount: 1.2,
                millis: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            };

            bench.iter(|| {
                let buffer = vec![];

                let mut writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_writer(buffer);

                (0..100)
                    .map(|i| OccurrenceId::new(i, occurrence.clone()))
                    .for_each(|s| writer.serialize(s).unwrap());
            });
        }

        #[bench]
        fn deserialize_occurrence(bench: &mut test::Bencher) {
            let data = (0..100)
                .map(|_| String::from("0\t1.2\t4\n"))
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
                    .deserialize::<Occurrence>()
                    .enumerate()
                    .for_each(|(_, s)| {
                        let s = s.unwrap();
                        assert_eq!(s.skull, 0);
                        assert_eq!(s.amount, 1.2);
                        assert_eq!(s.millis, 4);
                    })
            });
        }
    }
}
