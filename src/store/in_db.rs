#![allow(dead_code)]

use super::{
    crud::Response,
    data::{OccurrenceId, QuickId, SkullId},
    Crud, Data, Error, Id, Occurrence, Quick, Skull, Store,
};

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

pub struct InDb {
    users: std::collections::HashMap<String, std::sync::RwLock<sqlx::SqlitePool>>,
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

                let pool = sqlx::SqlitePool::connect_lazy(
                    format!("sqlite://{}", path.display()).as_str(),
                )?;

                log::info!("Allowing {user}");

                Ok((user, std::sync::RwLock::new(pool)))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { users })
    }
}

impl Store for InDb {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error> {
        let lock = self
            .users
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }
}

#[async_trait::async_trait]
impl<D: SqlData> Crud<D> for std::sync::RwLock<sqlx::SqlitePool> {
    async fn list(&self, limit: Option<u32>) -> Response<Vec<D::Id>> {
        let pool = self.read()?.clone();
        let data = D::list(limit, &pool).await?;
        let last_modified = D::last_modified(&pool).await?;
        Ok((data, last_modified))
    }

    async fn create(&self, data: D) -> Response<Id> {
        let pool = self.read()?.clone();
        let data = D::create(data, &pool).await?;
        let last_modified = D::last_modified(&pool).await?;
        Ok((data, last_modified))
    }

    async fn read(&self, id: Id) -> Response<D::Id> {
        let pool = self.read()?.clone();
        let data = D::read(id, &pool).await?;
        let last_modified = D::last_modified(&pool).await?;
        Ok((data, last_modified))
    }

    async fn update(&self, id: Id, data: D) -> Response<D::Id> {
        let pool = self.read()?.clone();
        let data = D::update(data, id, &pool).await?;
        let last_modified = D::last_modified(&pool).await?;
        Ok((data, last_modified))
    }

    async fn delete(&self, id: Id) -> Response<D::Id> {
        let pool = self.read()?.clone();
        let data = D::delete(id, &pool).await?;
        let last_modified = D::last_modified(&pool).await?;
        Ok((data, last_modified))
    }

    async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let pool = self.read()?.clone();
        D::last_modified(&pool).await
    }
}

#[async_trait::async_trait]
trait SqlData: Data + 'static {
    async fn list(limit: Option<u32>, pool: &sqlx::SqlitePool) -> Result<Vec<Self::Id>, Error>;
    async fn create(self, pool: &sqlx::SqlitePool) -> Result<Id, Error>;
    async fn read(id: Id, pool: &sqlx::SqlitePool) -> Result<Self::Id, Error>;
    async fn update(self, id: Id, pool: &sqlx::SqlitePool) -> Result<Self::Id, Error>;
    async fn delete(id: Id, pool: &sqlx::SqlitePool) -> Result<Self::Id, Error>;
    async fn last_modified(pool: &sqlx::SqlitePool) -> Result<std::time::SystemTime, Error>;
}

#[async_trait::async_trait]
impl SqlData for Skull {
    async fn list(limit: Option<u32>, pool: &sqlx::SqlitePool) -> Result<Vec<SkullId>, Error> {
        if let Some(limit) = limit {
            sqlx::query_as!(
                SkullId,
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
                "#,
                limit
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                SkullId,
                r#"
                SELECT
                    "id" as "id: _",
                    "name",
                    "color",
                    "icon",
                    "unit_price" as "unit_price: _",
                    "limit" as "limit: _"
                FROM skulls
                "#
            )
            .fetch_all(pool)
            .await
        }
        .map_err(Into::into)
    }

    async fn create(self, pool: &sqlx::SqlitePool) -> Result<Id, Error> {
        sqlx::query_as!(
            transient::Id,
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    async fn read(id: Id, pool: &sqlx::SqlitePool) -> Result<SkullId, Error> {
        sqlx::query_as!(
            SkullId,
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
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn update(self, id: Id, pool: &sqlx::SqlitePool) -> Result<SkullId, Error> {
        let mut tx = pool.begin().await?;

        let previous = sqlx::query_as!(
            SkullId,
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
            id
        )
        .fetch_optional(&mut tx)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))?;

        sqlx::query!(
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
            id,
            self.name,
            self.color,
            self.icon,
            self.unit_price,
            self.limit,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(previous)
    }

    async fn delete(id: Id, pool: &sqlx::SqlitePool) -> Result<SkullId, Error> {
        sqlx::query_as!(
            SkullId,
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
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn last_modified(pool: &sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[async_trait::async_trait]
impl SqlData for Quick {
    async fn list(limit: Option<u32>, pool: &sqlx::SqlitePool) -> Result<Vec<QuickId>, Error> {
        if let Some(limit) = limit {
            sqlx::query_as!(
                QuickId,
                r#"
                SELECT
                    "id" as "id: _",
                    "skull" as "skull: _",
                    "amount" as "amount: _"
                FROM quicks
                LIMIT $1
                "#,
                limit
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                QuickId,
                r#"
                SELECT
                    "id" as "id: _",
                    "skull" as "skull: _",
                    "amount" as "amount: _"
                FROM quicks
                "#
            )
            .fetch_all(pool)
            .await
        }
        .map_err(Into::into)
    }

    async fn create(self, pool: &sqlx::SqlitePool) -> Result<Id, Error> {
        sqlx::query_as!(
            transient::Id,
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    async fn read(id: Id, pool: &sqlx::SqlitePool) -> Result<QuickId, Error> {
        sqlx::query_as!(
            QuickId,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            FROM quicks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn update(self, id: Id, pool: &sqlx::SqlitePool) -> Result<QuickId, Error> {
        let mut tx = pool.begin().await?;

        let previous = sqlx::query_as!(
            QuickId,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            FROM quicks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut tx)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))?;

        sqlx::query!(
            r#"
            UPDATE quicks
            SET
                "skull" = $2,
                "amount" = $3
            WHERE id = $1
            "#,
            id,
            self.skull,
            self.amount,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(previous)
    }

    async fn delete(id: Id, pool: &sqlx::SqlitePool) -> Result<QuickId, Error> {
        sqlx::query_as!(
            QuickId,
            r#"
            DELETE FROM quicks
            WHERE id = $1
            RETURNING
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _"
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn last_modified(pool: &sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[async_trait::async_trait]
impl SqlData for Occurrence {
    async fn list(limit: Option<u32>, pool: &sqlx::SqlitePool) -> Result<Vec<OccurrenceId>, Error> {
        if let Some(limit) = limit {
            sqlx::query_as!(
                OccurrenceId,
                r#"
                SELECT
                    "id" as "id!: _",
                    "skull" as "skull!: _",
                    "amount" as "amount!: _",
                    "millis" as "millis!: _"
                FROM occurrences
                ORDER BY "millis" DESC, "id" DESC
                LIMIT $1
                "#,
                limit
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                OccurrenceId,
                r#"
                SELECT
                    "id" as "id: _",
                    "skull" as "skull: _",
                    "amount" as "amount: _",
                    "millis" as "millis: _"
                FROM occurrences
                ORDER BY "millis" DESC, "id" DESC
                "#
            )
            .fetch_all(pool)
            .await
        }
        .map_err(Into::into)
    }

    async fn create(self, pool: &sqlx::SqlitePool) -> Result<Id, Error> {
        sqlx::query_as!(
            transient::Id,
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    async fn read(id: Id, pool: &sqlx::SqlitePool) -> Result<OccurrenceId, Error> {
        sqlx::query_as!(
            OccurrenceId,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            FROM occurrences
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn update(self, id: Id, pool: &sqlx::SqlitePool) -> Result<OccurrenceId, Error> {
        let mut tx = pool.begin().await?;

        let previous = sqlx::query_as!(
            OccurrenceId,
            r#"
            SELECT
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            FROM occurrences
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut tx)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))?;

        sqlx::query!(
            r#"
            UPDATE occurrences
            SET
                "skull" = $2,
                "amount" = $3,
                "millis" = $4
            WHERE id = $1
            "#,
            id,
            self.skull,
            self.amount,
            self.millis,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(previous)
    }

    async fn delete(id: Id, pool: &sqlx::SqlitePool) -> Result<OccurrenceId, Error> {
        sqlx::query_as!(
            OccurrenceId,
            r#"
            DELETE FROM occurrences
            WHERE id = $1
            RETURNING
                "id" as "id: _",
                "skull" as "skull: _",
                "amount" as "amount: _",
                "millis"
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    async fn last_modified(pool: &sqlx::SqlitePool) -> Result<std::time::SystemTime, Error> {
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
        .fetch_one(pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

#[cfg(test)]
mod test {
    use crate::{store::test::USER, test_util::TestPath};

    use super::{InDb, Store};

    struct TestStore(InDb, crate::test_util::TestPath);

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
        fn skull(
            &self,
            user: &str,
        ) -> Result<&dyn crate::store::Crud<crate::store::Skull>, crate::store::Error> {
            self.0.skull(user)
        }

        fn quick(
            &self,
            user: &str,
        ) -> Result<&dyn crate::store::Crud<crate::store::Quick>, crate::store::Error> {
            self.0.quick(user)
        }

        fn occurrence(
            &self,
            user: &str,
        ) -> Result<&dyn crate::store::Crud<crate::store::Occurrence>, crate::store::Error>
        {
            self.0.occurrence(user)
        }
    }

    crate::impl_crud_tests!(InDb, TestStore::new().await);
}
