#![allow(dead_code)]

use super::{
    data::{OccurrenceId, QuickId, SkullId},
    Crud, Data, Error, Id, Occurrence, Quick, Skull, Store,
};

mod transient {
    pub(super) struct Id {
        pub id: super::Id,
    }

    pub(super) struct Time {
        pub millis: Option<i64>,
    }

    impl Time {
        pub(super) fn unpack(
            maybe_self: Option<Self>,
        ) -> Result<std::time::SystemTime, super::Error> {
            if let Some(millis) = maybe_self.and_then(|s| s.millis) {
                let millis = std::time::Duration::from_millis(millis.try_into()?);
                Ok(std::time::UNIX_EPOCH + millis)
            } else {
                Ok(std::time::UNIX_EPOCH)
            }
        }
    }
}

pub struct InDb(std::collections::HashMap<String, std::sync::RwLock<sqlx::SqlitePool>>);

impl Store for InDb {
    fn skull(&self, user: &str) -> Result<&dyn Crud<Skull>, Error> {
        let lock = self
            .0
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn quick(&self, user: &str) -> Result<&dyn Crud<Quick>, Error> {
        let lock = self
            .0
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }

    fn occurrence(&self, user: &str) -> Result<&dyn Crud<Occurrence>, Error> {
        let lock = self
            .0
            .get(user)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))?;
        Ok(lock)
    }
}

#[async_trait::async_trait]
impl<D: SqlData> Crud<D> for std::sync::RwLock<sqlx::SqlitePool> {
    async fn list(&self, limit: Option<u32>) -> Result<Vec<D::Id>, Error> {
        let pool = self.read()?.clone();
        D::list(limit, &pool).await
    }

    async fn create(&self, data: D) -> Result<Id, Error> {
        let pool = self.read()?.clone();
        D::create(data, &pool).await
    }

    async fn read(&self, id: Id) -> Result<D::Id, Error> {
        let pool = self.read()?.clone();
        D::read(id, &pool).await
    }

    async fn update(&self, id: Id, data: D) -> Result<D::Id, Error> {
        let pool = self.read()?.clone();
        D::update(data, id, &pool).await
    }

    async fn delete(&self, id: Id) -> Result<D::Id, Error> {
        let pool = self.read()?.clone();
        D::delete(id, &pool).await
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
        sqlx::query_as!(
            SkullId,
            r#"
            UPDATE skulls
            SET
                "name" = $2,
                "color" = $3,
                "icon" = $4,
                "unit_price" = $5,
                "limit" = $6
            WHERE id = $1
            RETURNING
                "id" as "id!: _",
                "name" as "name!",
                "color" as "color!",
                "icon" as "icon!",
                "unit_price" as "unit_price!: _",
                "limit" as "limit: _"
            "#,
            id,
            self.name,
            self.color,
            self.icon,
            self.unit_price,
            self.limit,
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
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
        .fetch_optional(pool)
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
        sqlx::query_as!(
            QuickId,
            r#"
            UPDATE quicks
            SET
                "skull" = $2,
                "amount" = $3
            WHERE id = $1
            RETURNING
                "id" as "id!: _",
                "skull" as "skull!: _",
                "amount" as "amount!: _"
            "#,
            id,
            self.skull,
            self.amount,
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
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
        .fetch_optional(pool)
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
                    "id" as "id: _",
                    "skull" as "skull: _",
                    "amount" as "amount: _",
                    "millis" as "millis: _"
                FROM occurrences
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
        sqlx::query_as!(
            OccurrenceId,
            r#"
            UPDATE occurrences
            SET
                "skull" = $2,
                "amount" = $3,
                "millis" = $4
            WHERE id = $1
            RETURNING
                "id" as "id!: _",
                "skull" as "skull!: _",
                "amount" as "amount!: _",
                "millis" as "millis!: _"
            "#,
            id,
            self.skull,
            self.amount,
            self.millis,
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
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
                "table" = 1
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}
