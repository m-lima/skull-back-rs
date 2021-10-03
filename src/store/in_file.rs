use super::{Crud, Data, Error, Id, Occurrence, Quick, Skull, Store};

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
                    std::fs::File::create(&file).map_err(|e| {
                        anyhow::anyhow!("Could not create {}: {}", file.display(), e)
                    })?;
                } else if file.is_dir() {
                    anyhow::bail!("Path {} is not a file", file.display());
                }
            }
        }

        log::info!(
            "Allowing users [{}]",
            users
                .iter()
                .map(Clone::clone)
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(Self { path, users })
    }
}

impl Store for InFile {
    fn last_modified(&self, user: &str) -> Result<std::time::SystemTime, Error> {
        let path = self.path.join(user);
        let skull = std::fs::metadata(path.join("skull")).unwrap().modified()?;
        let quick = std::fs::metadata(path.join("quick")).unwrap().modified()?;
        let occurrence = std::fs::metadata(path.join("occurrence"))
            .unwrap()
            .modified()?;

        Ok(std::cmp::max(skull, std::cmp::max(quick, occurrence)))
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

impl InFile {
    fn validate(&self, user: &str) -> Result<(), Error> {
        if self.users.contains(user) {
            Ok(())
        } else {
            Err(Error::NoSuchUser(String::from(user)))
        }
    }

    fn reader<D: Named>(&self, user: &str) -> Result<csv::Reader<std::fs::File>, Error> {
        self.validate(user)?;
        let file = std::fs::File::open(self.path.join(user).join(D::name()))?;
        Ok(csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(file))
    }

    fn load<D: Named>(&self, user: &str) -> Result<Vec<D>, Error> {
        let mut reader = self.reader::<D>(user)?;

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

    fn write<D: Data>(path: std::path::PathBuf, entries: Vec<D>) -> Result<(), Error> {
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
            .write(true)
            .open(path)?
            .write_all(
                writer
                    .into_inner()
                    .map_err(|e| Error::Serde(e.to_string()))?
                    .as_slice(),
            )
            .map_err(Error::Io)
    }
}

impl<D: Named> Crud<D> for InFile {
    fn list(&self, user: &str) -> Result<Vec<std::borrow::Cow<'_, D>>, Error> {
        let mut reader = self.reader::<D>(user)?;

        Ok(reader
            .deserialize()
            .filter_map(Result::ok)
            .map(std::borrow::Cow::Owned)
            .collect())
    }

    fn filter_list(
        &self,
        user: &str,
        filter: Box<dyn Fn(&D) -> bool>,
    ) -> Result<Vec<std::borrow::Cow<'_, D>>, Error> {
        let mut reader = self.reader::<D>(user)?;

        Ok(reader
            .deserialize()
            .filter_map(Result::ok)
            .filter(|d| (filter)(d))
            .map(std::borrow::Cow::Owned)
            .collect())
    }

    fn create(&mut self, user: &str, mut data: D) -> Result<Id, Error> {
        let id = self.reader::<D>(user).map(|mut reader| {
            reader
                .deserialize::<D>()
                .filter_map(Result::ok)
                .last()
                .map_or(0, |d| d.id() + 1)
        })?;

        data.set_id(id);

        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(self.path.join(user).join(D::name()))?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_writer(file);
        writer.serialize(data).map_err(|e| Error::Io(e.into()))?;
        Ok(id)
    }

    fn read(&self, user: &str, id: Id) -> Result<std::borrow::Cow<'_, D>, Error> {
        let mut reader = self.reader::<D>(user)?;

        reader
            .deserialize()
            .filter_map(Result::ok)
            .find(|d| Data::id(d) == id)
            .map(std::borrow::Cow::Owned)
            .ok_or(Error::NotFound(id))
    }

    fn update(&mut self, user: &str, id: Id, mut data: D) -> Result<D, Error> {
        let mut entries = self.load::<D>(user)?;

        let index = find(id, &entries).ok_or(Error::NotFound(id))?;
        let old = &mut entries[index];
        data.set_id(old.id());
        std::mem::swap(old, &mut data);

        Self::write(self.path.join(user).join(D::name()), entries)?;

        Ok(data)
    }

    fn delete(&mut self, user: &str, id: Id) -> Result<D, Error> {
        let mut entries = self.load::<D>(user)?;

        let index = find(id, &entries).ok_or(Error::NotFound(id))?;
        let data = entries.remove(index);

        Self::write(self.path.join(user).join(D::name()), entries)?;

        Ok(data)
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
