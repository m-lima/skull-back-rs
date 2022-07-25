use super::{crud::Response, Crud, Data, Error, Id, Occurrence, Quick, Skull, Store, WithId};

#[cfg(all(test, nightly))]
mod bench;
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
    pub fn new(
        users: std::collections::HashMap<String, std::path::PathBuf>,
    ) -> anyhow::Result<Self> {
        let users = users
            .into_iter()
            .map(|(user, path)| {
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
                        std::fs::File::create(&file).map_err(|e| {
                            anyhow::anyhow!("Could not create {}: {e}", file.display())
                        })?;
                    } else if file.is_dir() {
                        anyhow::bail!("Path {} is not a file", file.display());
                    }
                }

                log::info!("Allowing {user}");

                let skull = std::sync::RwLock::new(UserFile::new(path.join(Skull::name())));
                let quick = std::sync::RwLock::new(UserFile::new(path.join(Quick::name())));
                let occurrence =
                    std::sync::RwLock::new(UserFile::new(path.join(Occurrence::name())));
                Ok((
                    user,
                    UserStore {
                        skull,
                        quick,
                        occurrence,
                    },
                ))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self { users })
    }
}

impl Store for InFile {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error> {
        let user_file = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_file)
    }

    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error> {
        let user_file = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_file)
    }

    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error> {
        let user_file = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(user_file)
    }
}

struct UserStore {
    skull: std::sync::RwLock<UserFile<Skull>>,
    quick: std::sync::RwLock<UserFile<Quick>>,
    occurrence: std::sync::RwLock<UserFile<Occurrence>>,
}

#[async_trait::async_trait]
impl<D: FileData> Crud<D> for UserStore {
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

struct UserFile<D: Data> {
    file: std::path::PathBuf,
    next_id: u32,
    _marker: std::marker::PhantomData<D>,
}

impl<D: FileData> UserFile<D> {
    fn new(file: std::path::PathBuf) -> Self {
        Self {
            file,
            next_id: 1,
            _marker: std::marker::PhantomData,
        }
    }

    fn lines(&self) -> Result<impl Iterator<Item = D::Id> + '_, Error> {
        use std::io::BufRead;
        let iter = std::io::BufReader::new(std::fs::File::open(&self.file)?)
            .lines()
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| match line.1 {
                Ok(line) => Some(line),
                Err(err) => {
                    log::error!("Failed to read {}:{}: {err}", self.file.display(), line.0);
                    None
                }
            });
        Ok(iter)
    }

    fn ids(&self) -> Result<impl Iterator<Item = Result<Id, Error>>, Error> {
        use std::io::BufRead;
        let iter = std::io::BufReader::new(std::fs::File::open(&self.file)?)
            .lines()
            .map(D::id);
        Ok(iter)
    }

    fn append(&mut self, data: D) -> Result<(Id, std::time::SystemTime), Error> {
        if self.next_id == u32::MAX {
            return Err(Error::StoreFull);
        }
        let mut file = std::fs::File::options().append(true).open(&self.file)?;
        let id = self.next_id;
        D::write_tsv(D::Id::new(id, data), &mut file)?;
        self.next_id += 1;

        Ok((id, self.last_modified()?))
    }

    fn replace(&mut self, entries: Vec<D::Id>) -> Result<std::time::SystemTime, Error> {
        use std::io::Write;

        let mut buffer = vec![];
        for entry in entries {
            D::write_tsv(entry, &mut buffer)?;
        }

        std::fs::File::options()
            .truncate(true)
            .write(true)
            .open(&self.file)?
            .write_all(buffer.as_slice())?;

        self.last_modified()
    }

    fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        std::fs::metadata(&self.file)
            .and_then(|f| f.modified())
            .map_err(Error::Io)
    }
}

trait FileData: Serializable + 'static {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserFile<Self>>;
    fn create(store: &UserStore, data: Self) -> Response<Id>;
    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id>;
    fn delete(store: &UserStore, id: Id) -> Response<Self::Id>;
    fn conflicts(&self, other: &Self::Id) -> bool;

    fn list(store: &UserStore, limit: Option<u32>) -> Response<Vec<Self::Id>> {
        let lock = Self::as_read(store)?;

        let entries = lock.lines()?.collect::<Vec<_>>();

        if let Some(limit) = limit.map(usize::try_from).and_then(Result::ok) {
            let len = entries.len();
            Ok((
                entries.into_iter().skip(len - limit.min(len)).collect(),
                lock.last_modified()?,
            ))
        } else {
            Ok((entries, lock.last_modified()?))
        }
    }

    fn read(store: &UserStore, id: Id) -> Response<Self::Id> {
        let lock = Self::as_read(store)?;
        let data = lock
            .lines()?
            .find(|d| d.id() == id)
            .ok_or(Error::NotFound(id))?;
        Ok((data, lock.last_modified()?))
    }

    fn last_modified(store: &UserStore) -> Result<std::time::SystemTime, Error> {
        Self::as_read(store)?.last_modified()
    }

    fn update_inner(
        store: &UserStore,
        index: usize,
        mut entries: Vec<Self::Id>,
        mut data: Self::Id,
    ) -> Response<Self::Id> {
        let left = &mut entries[index];
        let last_modified = if left == &data {
            Self::as_read(store)?.last_modified()
        } else {
            std::mem::swap(left, &mut data);
            Self::as_write(store)?.replace(entries)
        }?;
        Ok((data, last_modified))
    }

    fn delete_inner(store: &UserStore, id: Id) -> Response<Self::Id> {
        let (index, mut entries) = Self::entries_with_index(store, id)?;

        let old = entries.remove(index);

        let last_modified = Self::as_write(store)?.replace(entries)?;
        Ok((old, last_modified))
    }

    fn has_skull(store: &UserStore, skull: Id) -> Result<(), Error> {
        Skull::as_read(store)?
            .ids()?
            .filter_map(Result::ok)
            .find(|id| *id == skull)
            .map(|_| ())
            .ok_or(Error::Constraint)
    }

    fn entries_with_index(store: &UserStore, id: Id) -> Result<(usize, Vec<Self::Id>), Error> {
        let mut index = None;
        let entries = Self::as_read(store)?
            .lines()?
            .enumerate()
            .map(|line| {
                if line.1.id() == id {
                    index = Some(line.0);
                }
                line.1
            })
            .collect::<Vec<_>>();
        index.map(|i| (i, entries)).ok_or(Error::NotFound(id))
    }

    fn as_read(store: &UserStore) -> Result<std::sync::RwLockReadGuard<'_, UserFile<Self>>, Error> {
        Ok(Self::get(store).read()?)
    }

    fn as_write(
        store: &UserStore,
    ) -> Result<std::sync::RwLockWriteGuard<'_, UserFile<Self>>, Error> {
        Ok(Self::get(store).write()?)
    }
}

impl FileData for Skull {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserFile<Self>> {
        &store.skull
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        if Self::as_read(store)?.lines()?.any(|d| data.conflicts(&d)) {
            Err(Error::Constraint)
        } else {
            Self::as_write(store)?.append(data)
        }
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let (index, entries) = Self::entries_with_index(store, id)?;

        if entries
            .iter()
            .filter(|d| d.id != id)
            .any(|d| data.conflicts(d))
        {
            Err(Error::Constraint)
        } else {
            Self::update_inner(store, index, entries, Self::Id::new(id, data))
        }
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        if Occurrence::as_read(store)?.lines()?.any(|d| d.skull == id) {
            Err(Error::Constraint)
        } else {
            let quicks = Quick::as_read(store)?
                .lines()?
                .filter(|d| d.skull != id)
                .collect();

            let mut quick_lock = Quick::as_write(store)?;
            let (old, last_modified) = {
                let (index, mut entries) = Self::entries_with_index(store, id)?;
                let old = entries.remove(index);
                let last_modified = Self::as_write(store)?.replace(entries)?;
                (old, last_modified)
            };
            quick_lock.replace(quicks)?;

            Ok((old, last_modified))
        }
    }

    fn conflicts(&self, other: &Self::Id) -> bool {
        self.name == other.name || self.color == other.color || self.icon == other.icon
    }
}

impl FileData for Quick {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserFile<Self>> {
        &store.quick
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        Self::has_skull(store, data.skull)?;
        if Self::as_read(store)?.lines()?.any(|d| data.conflicts(&d)) {
            Err(Error::Constraint)
        } else {
            Self::as_write(store)?.append(data)
        }
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let (index, entries) = Self::entries_with_index(store, id)?;
        Self::has_skull(store, data.skull)?;
        if entries
            .iter()
            .filter(|d| d.id != id)
            .any(|d| data.conflicts(d))
        {
            Err(Error::Constraint)
        } else {
            Self::update_inner(store, index, entries, Self::Id::new(id, data))
        }
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        Self::delete_inner(store, id)
    }

    fn conflicts(&self, other: &Self::Id) -> bool {
        self.skull == other.skull && (self.amount - other.amount).abs() < f32::EPSILON
    }
}

impl FileData for Occurrence {
    fn get(store: &UserStore) -> &std::sync::RwLock<UserFile<Self>> {
        &store.occurrence
    }

    fn create(store: &UserStore, data: Self) -> Response<Id> {
        Self::has_skull(store, data.skull)?;
        Self::as_write(store)?.append(data)
    }

    fn update(store: &UserStore, id: Id, data: Self) -> Response<Self::Id> {
        let (index, entries) = Self::entries_with_index(store, id)?;
        Self::has_skull(store, data.skull)?;
        Self::update_inner(store, index, entries, Self::Id::new(id, data))
    }

    fn delete(store: &UserStore, id: Id) -> Response<Self::Id> {
        Self::delete_inner(store, id)
    }

    fn conflicts(&self, _other: &Self::Id) -> bool {
        false
    }
}

trait Serializable: Data {
    fn name() -> &'static str;
    fn id(string: std::io::Result<String>) -> Result<Id, Error>;
    fn read_tsv(string: std::io::Result<String>) -> Result<Self::Id, Error>;
    fn write_tsv<W: std::io::Write>(with_id: Self::Id, writer: &mut W) -> Result<(), Error>;
}

impl Serializable for Skull {
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

impl Serializable for Quick {
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

impl Serializable for Occurrence {
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
    use crate::{
        store::{data::SkullId, test::USER, WithId},
        test_util::{create_base_test_path, TestPath},
    };

    use super::{Data, Error, FileData, InFile, Serializable, Skull, Store, UserFile, UserStore};

    crate::impl_crud_tests!(InFile, TestStore::new());

    struct TestStore {
        store: InFile,
        _path: TestPath,
    }

    impl TestStore {
        fn new() -> Self {
            let path = create_base_test_path();
            let store = InFile::new(
                Some((String::from(USER), path.join(USER)))
                    .into_iter()
                    .collect(),
            )
            .unwrap();

            Self { store, _path: path }
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

    #[test]
    fn create_store_full() {
        fn full_container<D: Data>() -> std::sync::RwLock<UserFile<D>> {
            std::sync::RwLock::new(UserFile {
                next_id: u32::MAX,
                file: std::path::PathBuf::new(),
                _marker: std::marker::PhantomData,
            })
        }

        let store = UserStore {
            skull: full_container(),
            quick: full_container(),
            occurrence: full_container(),
        };

        let skull = Skull {
            name: String::from("skull"),
            color: String::from("red"),
            icon: String::new(),
            unit_price: 1.,
            limit: None,
        };

        assert_eq!(
            Skull::create(&store, skull).unwrap_err().to_string(),
            Error::StoreFull.to_string()
        );
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

        let skull = SkullId::new(
            0,
            Skull {
                name: String::from("skull"),
                color: String::from('0'),
                icon: String::new(),
                unit_price: 1.,
                limit: None,
            },
        );
        let mut writer = FailedWriter;
        assert_eq!(
            Skull::write_tsv(skull, &mut writer)
                .unwrap_err()
                .to_string(),
            String::from("Serde error: Could not serialize `id` for Skull: Serde error: write")
        );
    }
}
