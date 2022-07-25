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
    async fn list(&self, limit: Option<u32>) -> Response<Vec<D::Id>> {
        let lock = self.read()?;

        let entries = lock
            .lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .collect::<Vec<_>>();

        if let Some(limit) = limit.map(usize::try_from).and_then(Result::ok) {
            let len = entries.len();
            Ok((
                entries.into_iter().skip(len - limit).collect(),
                last_modified(&lock.file)?,
            ))
        } else {
            Ok((entries, last_modified(&lock.file)?))
        }
    }

    async fn create(&self, data: D) -> Response<Id> {
        let lock = self.write()?;
        let id = lock
            .lines()?
            .map(D::id)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .max()
            .map_or(1, |id| id + 1);

        let mut file = std::fs::File::options().append(true).open(&lock.file)?;
        D::write_tsv(D::Id::new(id, data), &mut file)?;

        Ok((id, last_modified(&lock.file)?))
    }

    async fn read(&self, id: Id) -> Response<D::Id> {
        let lock = self.read()?;
        let data = lock
            .lines()?
            .map(D::read_tsv)
            .enumerate()
            .filter_map(|line| lock.good_line(line))
            .find(|d| d.id() == id)
            .ok_or(Error::NotFound(id))?;
        Ok((data, last_modified(&lock.file)?))
    }

    async fn update(&self, id: Id, data: D) -> Response<D::Id> {
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
        Ok((new, last_modified(&lock.file)?))
    }

    async fn delete(&self, id: Id) -> Response<D::Id> {
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
        Ok((old, last_modified(&lock.file)?))
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let lock = self.read()?;
        std::fs::metadata(&lock.file)
            .and_then(|f| f.modified())
            .map_err(Error::Io)
    }
}

fn last_modified(path: &std::path::Path) -> Result<std::time::SystemTime, Error> {
    std::fs::metadata(path)
        .and_then(|f| f.modified())
        .map_err(Error::Io)
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
    use crate::{
        store::{data::SkullId, test::USER, WithId},
        test_util::{create_base_test_path, TestPath},
    };

    use super::{Error, FileData, InFile, Skull, Store};

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
