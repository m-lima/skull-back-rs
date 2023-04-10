use super::{crud::Response, Crud, Data, Error, Id, Model, Occurrence, Quick, Skull, Store};

mod transient {
    pub(super) struct Id {
        pub id: super::Id,
    }

    pub(super) struct Time {
        pub millis: i64,
    }

    impl Time {
        pub(super) fn unpack(self) -> Result<std::time::SystemTime, super::Error> {
            let millis = std::time::Duration::from_millis(self.millis.try_into()?);
            Ok(std::time::UNIX_EPOCH + millis)
        }
    }
}

pub struct Pools {
    read: std::sync::RwLock<sqlx::SqlitePool>,
    write: std::sync::RwLock<sqlx::SqlitePool>,
}

pub struct InDb {
    users: std::collections::HashMap<String, Pools>,
}

impl InDb {
    pub fn new(
        users: std::collections::HashMap<String, std::path::PathBuf>,
    ) -> anyhow::Result<Self> {
        let users = users
            .into_iter()
            .map(|(user, path)| {
                if !path.exists() {
                    log::debug!("Creating {}", path.display());
                    std::fs::File::create(&path).map_err(|e| {
                        anyhow::anyhow!("Could not create user database {}: {e}", path.display())
                    })?;
                } else if !path.is_file() {
                    anyhow::bail!("User path is not a file {}", path.display());
                }

                let options = format!("sqlite://{}", path.display())
                    .parse::<sqlx::sqlite::SqliteConnectOptions>()?
                    .pragma("query_only", "true")
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
                let read_pool = sqlx::SqlitePool::connect_lazy_with(options);

                let options = format!("sqlite://{}", path.display())
                    .parse::<sqlx::sqlite::SqliteConnectOptions>()?
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
                let write_pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_lazy_with(options);

                log::info!("Allowing {user}");

                Ok((
                    user,
                    Pools {
                        read: std::sync::RwLock::new(read_pool),
                        write: std::sync::RwLock::new(write_pool),
                    },
                ))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { users })
    }
}

impl Store for InDb {
    type Crud<M: Model> = Pools;

    fn skull(&self, user: &str) -> Result<&Self::Crud<Skull>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn quick(&self, user: &str) -> Result<&Self::Crud<Quick>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn occurrence(&self, user: &str) -> Result<&Self::Crud<Occurrence>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }
}

// This RwLock is doing nothing besides allowing us to use it with Gotham
//
// The RAII lock guard gets immediately dropped and barely counts. However it allows the Store to
// implement RefUnwindSafe to be used as a state
macro_rules! get_pool {
    ($pool: ident, $lock: ident) => {
        match $pool.$lock.read().map_err(Error::from) {
            Ok(pool) => pool.clone(),
            Err(err) => return Box::pin(async { Err(err) }),
        }
    };
}

impl<D: SqlData> Crud<D> for Pools {
    type Future<T: Send + Unpin> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

    fn list(&self, limit: Option<u32>) -> Self::Future<Response<Vec<D::Id>>> {
        let pool = get_pool!(self, read);
        D::list(limit, pool)
    }

    fn create(&self, data: D) -> Self::Future<Response<Id>> {
        let pool = get_pool!(self, write);
        D::create(data, pool)
    }

    fn read(&self, id: Id) -> Self::Future<Response<D::Id>> {
        let pool = get_pool!(self, read);
        D::read(id, pool)
    }

    fn update(&self, id: Id, data: D) -> Self::Future<Response<D::Id>> {
        let pool = get_pool!(self, write);
        D::update(data, id, pool)
    }

    fn delete(&self, id: Id) -> Self::Future<Response<D::Id>> {
        let pool = get_pool!(self, write);
        D::delete(id, pool)
    }

    fn last_modified(&self) -> Self::Future<Result<std::time::SystemTime, Error>> {
        let pool = get_pool!(self, read);
        D::last_modified(pool)
    }
}

#[async_trait::async_trait]
pub trait SqlData: Data + 'static {
    const TABLE_ID: u32;
    async fn list(limit: Option<u32>, pool: sqlx::SqlitePool) -> Response<Vec<Self::Id>>;
    async fn create(self, pool: sqlx::SqlitePool) -> Response<Id>;
    async fn read(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id>;
    async fn update(self, id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id>;
    async fn delete(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id>;
    async fn last_modified(pool: sqlx::SqlitePool) -> Result<std::time::SystemTime, Error>;
}

macro_rules! query {
    (list, $limit:expr, $pool:tt, $data:path, $query:literal) => {{
        type DataId = <$data as Data>::Id;

        let mut conn = $pool.acquire().await?;
        let limit = $limit.map(i64::from).unwrap_or(-1);
        let data = sqlx::query_as!(DataId, $query, limit)
            .fetch_all(&mut conn)
            .await?;
        Ok((data, query!(last_modified, conn, $data)))
    }};

    (create, $pool:tt, $data:path, $query:literal, $($fields:tt)*) => {{
        let mut conn = $pool.acquire().await?;
        let data = sqlx::query_as!(
            transient::Id,
            $query,
            $($fields)*
        )
        .fetch_one(&mut conn)
        .await
        .map(|id| id.id)?;
        Ok((data, query!(last_modified, conn, $data)))
    }};

    (read, $id:tt, $pool:tt, $data:path, $query:literal) => {{
        type DataId = <$data as Data>::Id;

        let mut conn = $pool.acquire().await?;
        let data = sqlx::query_as!(DataId, $query, $id)
            .fetch_optional(&mut conn)
            .await
            .map_err(Into::into)
            .and_then(|d| d.ok_or(Error::NotFound($id)))?;
        Ok((data, query!(last_modified, conn, $data)))
    }};

    (update, $self:tt, $id:tt, $pool:tt, $data:path, $previous:literal, $update:literal, $($fields:tt)*) => {{
        type DataId = <$data as Data>::Id;

        let mut tx = $pool.begin().await?;
        let data = sqlx::query_as!(
            DataId,
            $previous,
            $id
        )
        .fetch_optional(&mut tx)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound($id)))?;

        if data != $self {
            sqlx::query!($update, $id, $($fields)*)
            .execute(&mut tx)
            .await?;
        }

        let last_modified = query!(last_modified, tx, $data);
        tx.commit().await?;

        Ok((data, last_modified))
    }};

    (delete, $id:tt, $pool:tt, $data:path, $query:literal) => {{
        type DataId = <$data as Data>::Id;

        let mut conn = $pool.acquire().await?;
        let data = sqlx::query_as!(DataId, $query, $id)
            .fetch_optional(&mut conn)
            .await
            .map_err(Into::into)
            .and_then(|d| d.ok_or(Error::NotFound($id)))?;
        Ok((data, query!(last_modified, conn, $data)))
    }};


    (last_modified, $conn:expr, $data:path) => {{
        const TABLE_ID: u32 = <$data>::TABLE_ID;

        sqlx::query_as!(
            transient::Time,
            r#"SELECT "millis" FROM last_modified WHERE "table" = $1"#,
            TABLE_ID
        )
        .fetch_one(&mut $conn)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)?
    }};
}

#[async_trait::async_trait]
impl SqlData for Skull {
    const TABLE_ID: u32 = 0;

    async fn list(limit: Option<u32>, pool: sqlx::SqlitePool) -> Response<Vec<Self::Id>> {
        query!(
            list,
            limit,
            pool,
            Skull,
            r#"
            SELECT
                "id" as "id: _",
                "name",
                "color",
                "icon",
                "unit_price" as "unit_price: _",
                "limit" as "limit: _"
            FROM skulls
            LIMIT $1
            "#
        )
    }

    async fn create(self, pool: sqlx::SqlitePool) -> Response<Id> {
        query!(
            create,
            pool,
            Skull,
            r#"
            INSERT INTO skulls (
                "name",
                "color",
                "icon",
                "unit_price",
                "limit"
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            ) RETURNING
                "id" as "id: _"
            "#,
            self.name,
            self.color,
            self.icon,
            self.unit_price,
            self.limit,
        )
    }

    async fn read(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            read,
            id,
            pool,
            Skull,
            r#"
            SELECT
                "id" as "id: _",
                "name",
                "color",
                "icon",
                "unit_price" as "unit_price: _",
                "limit" as "limit: _"
            FROM skulls
            WHERE id = $1
            "#
        )
    }

    async fn update(self, id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            update,
            self,
            id,
            pool,
            Skull,
            r#"
            SELECT
                "id" as "id: _",
                "name",
                "color",
                "icon",
                "unit_price" as "unit_price: _",
                "limit" as "limit: _"
            FROM skulls
            WHERE id = $1
            "#,
            r#"
            UPDATE skulls
            SET
                "name" = $2,
                "color" = $3,
                "icon" = $4,
                "unit_price" = $5,
                "limit" = $6
            WHERE id = $1
            "#,
            self.name,
            self.color,
            self.icon,
            self.unit_price,
            self.limit,
        )
    }

    async fn delete(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            delete,
            id,
            pool,
            Skull,
            r#"
            DELETE FROM skulls
            WHERE id = $1
            RETURNING
                "id" as "id: _",
                "name",
                "color",
                "icon",
                "unit_price" as "unit_price: _",
                "limit" as "limit: _"
            "#
        )
    }

    async fn last_modified(pool: sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
        sqlx::query_as!(
            transient::Time,
            r#"
            SELECT
                "millis"
            FROM last_modified
            WHERE
                "table" = 0
            "#
        )
        .fetch_one(&pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[async_trait::async_trait]
impl SqlData for Quick {
    const TABLE_ID: u32 = 1;

    async fn list(limit: Option<u32>, pool: sqlx::SqlitePool) -> Response<Vec<Self::Id>> {
        query!(
            list,
            limit,
            pool,
            Quick,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            FROM quicks
            LIMIT $1
            "#
        )
    }

    async fn create(self, pool: sqlx::SqlitePool) -> Response<Id> {
        query!(
            create,
            pool,
            Quick,
            r#"
            INSERT INTO quicks (
                "skull",
                "amount"
            ) VALUES (
                $1,
                $2
            ) RETURNING
                "id" as "id: _"
            "#,
            self.skull,
            self.amount,
        )
    }

    async fn read(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            read,
            id,
            pool,
            Quick,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            FROM quicks
            WHERE id = $1
            "#
        )
    }

    async fn update(self, id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            update,
            self,
            id,
            pool,
            Quick,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            FROM quicks
            WHERE id = $1
            "#,
            r#"
            UPDATE quicks
            SET
                "skull" = $2,
                "amount" = $3
            WHERE id = $1
            "#,
            self.skull,
            self.amount,
        )
    }

    async fn delete(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            delete,
            id,
            pool,
            Quick,
            r#"
            DELETE FROM quicks
            WHERE id = $1
            RETURNING
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            "#
        )
    }

    async fn last_modified(pool: sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
        sqlx::query_as!(
            transient::Time,
            r#"
            SELECT
                "millis"
            FROM last_modified
            WHERE
                "table" = 1
            "#
        )
        .fetch_one(&pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[async_trait::async_trait]
impl SqlData for Occurrence {
    const TABLE_ID: u32 = 2;

    async fn list(limit: Option<u32>, pool: sqlx::SqlitePool) -> Response<Vec<Self::Id>> {
        query!(
            list,
            limit,
            pool,
            Occurrence,
            r#"
            SELECT
                "id" as "id!: _",
                "skull" as "skull!: _",
                "amount" as "amount!: _",
                "millis" as "millis!: _"
            FROM occurrences
            ORDER BY "millis" DESC, "id" DESC
            LIMIT $1
            "#
        )
    }

    async fn create(self, pool: sqlx::SqlitePool) -> Response<Id> {
        query!(
            create,
            pool,
            Occurrence,
            r#"
            INSERT INTO occurrences (
                "skull",
                "amount",
                "millis"
            ) VALUES (
                $1,
                $2,
                $3
            ) RETURNING
                "id" as "id: _"
            "#,
            self.skull,
            self.amount,
            self.millis,
        )
    }

    async fn read(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            read,
            id,
            pool,
            Occurrence,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            FROM occurrences
            WHERE id = $1
            "#
        )
    }

    async fn update(self, id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            update,
            self,
            id,
            pool,
            Occurrence,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            FROM occurrences
            WHERE id = $1
            "#,
            r#"
            UPDATE occurrences
            SET
                "skull" = $2,
                "amount" = $3,
                "millis" = $4
            WHERE id = $1
            "#,
            self.skull,
            self.amount,
            self.millis,
        )
    }

    async fn delete(id: Id, pool: sqlx::SqlitePool) -> Response<Self::Id> {
        query!(
            delete,
            id,
            pool,
            Occurrence,
            r#"
            DELETE FROM occurrences
            WHERE id = $1
            RETURNING
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            "#
        )
    }

    async fn last_modified(pool: sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
        sqlx::query_as!(
            transient::Time,
            r#"
            SELECT
                "millis"
            FROM last_modified
            WHERE
                "table" = 2
            "#
        )
        .fetch_one(&pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[cfg(test)]
mod test {
    use crate::store::Model;
    use test_utils::{TestPath, USER};

    use super::{InDb, Store};

    struct TestStore(InDb, TestPath);

    impl TestStore {
        async fn new() -> TestStore {
            let path = TestPath::new();

            let db = InDb::new(
                Some((String::from(USER), path.join(USER)))
                    .into_iter()
                    .collect(),
            )
            .unwrap();

            let pool =
                sqlx::SqlitePool::connect(&format!("sqlite://{}", path.join(USER).display()))
                    .await
                    .unwrap();
            sqlx::migrate!().run(&pool).await.unwrap();

            Self(db, path)
        }
    }

    impl Store for TestStore {
        type Crud<M: Model> = <InDb as Store>::Crud<M>;

        fn skull(&self, user: &str) -> Result<&Self::Crud<super::Skull>, crate::store::Error> {
            self.0.skull(user)
        }

        fn quick(&self, user: &str) -> Result<&Self::Crud<super::Quick>, crate::store::Error> {
            self.0.quick(user)
        }

        fn occurrence(
            &self,
            user: &str,
        ) -> Result<&Self::Crud<super::Occurrence>, crate::store::Error> {
            self.0.occurrence(user)
        }
    }

    crate::impl_crud_tests!(InDb, TestStore::new().await);
}
