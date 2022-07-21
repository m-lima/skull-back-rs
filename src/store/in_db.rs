#![allow(dead_code)]

use super::{
    data::{OccurrenceId, QuickId, SkullId},
    Data, Error, Id, Occurrence, Quick, Skull,
};

pub struct InDb(std::collections::HashMap<String, tokio::sync::RwLock<sqlx::SqlitePool>>);

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

impl InDb {
    fn skull(&self, user: &str) -> Result<UserTable<'_, Skull>, Error> {
        self.0
            .get(user)
            .map(UserTable::new)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
    }

    fn quick(&self, user: &str) -> Result<UserTable<'_, Quick>, Error> {
        self.0
            .get(user)
            .map(UserTable::new)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
    }

    fn occurrence(&self, user: &str) -> Result<UserTable<'_, Occurrence>, Error> {
        self.0
            .get(user)
            .map(UserTable::new)
            .ok_or_else(|| Error::NoSuchUser(String::from(user)))
    }
}

pub struct UserTable<'s, D: Data> {
    lock: &'s tokio::sync::RwLock<sqlx::SqlitePool>,
    _marker: std::marker::PhantomData<D>,
}

impl<'s, D: Data> UserTable<'s, D> {
    fn new(lock: &'s tokio::sync::RwLock<sqlx::SqlitePool>) -> Self {
        Self {
            lock,
            _marker: std::marker::PhantomData,
        }
    }
}

impl UserTable<'_, Skull> {
    pub async fn list(&self, limit: Option<u32>) -> Result<Vec<SkullId>, Error> {
        let read = self.lock.read().await;
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
            .fetch_all(&*read)
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
            .fetch_all(&*read)
            .await
        }
        .map_err(Into::into)
    }

    pub async fn create(&self, skull: Skull) -> Result<Id, Error> {
        let write = self.lock.write().await;
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
            skull.name,
            skull.color,
            skull.icon,
            skull.unit_price,
            skull.limit,
        )
        .fetch_one(&*write)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    pub async fn read(&self, id: Id) -> Result<SkullId, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn update(&self, id: Id, skull: Skull) -> Result<SkullId, Error> {
        let write = self.lock.write().await;
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
            skull.name,
            skull.color,
            skull.icon,
            skull.unit_price,
            skull.limit,
        )
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn delete(&mut self, id: Id) -> Result<SkullId, Error> {
        let write = self.lock.write().await;
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
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

impl UserTable<'_, Quick> {
    pub async fn list(&self, limit: Option<u32>) -> Result<Vec<QuickId>, Error> {
        let read = self.lock.read().await;
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
            .fetch_all(&*read)
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
            .fetch_all(&*read)
            .await
        }
        .map_err(Into::into)
    }

    pub async fn create(&self, quick: Quick) -> Result<Id, Error> {
        let write = self.lock.write().await;
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
            quick.skull,
            quick.amount,
        )
        .fetch_one(&*write)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    pub async fn read(&self, id: Id) -> Result<QuickId, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn update(&self, id: Id, quick: Quick) -> Result<QuickId, Error> {
        let write = self.lock.write().await;
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
            quick.skull,
            quick.amount,
        )
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn delete(&mut self, id: Id) -> Result<QuickId, Error> {
        let write = self.lock.write().await;
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
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}

impl UserTable<'_, Occurrence> {
    pub async fn list(&self, limit: Option<u32>) -> Result<Vec<OccurrenceId>, Error> {
        let read = self.lock.read().await;
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
            .fetch_all(&*read)
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
            .fetch_all(&*read)
            .await
        }
        .map_err(Into::into)
    }

    pub async fn create(&self, occurrence: Occurrence) -> Result<Id, Error> {
        let write = self.lock.write().await;
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
            occurrence.skull,
            occurrence.amount,
            occurrence.millis,
        )
        .fetch_one(&*write)
        .await
        .map_err(Into::into)
        .map(|id| id.id)
    }

    pub async fn read(&self, id: Id) -> Result<OccurrenceId, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn update(&self, id: Id, occurrence: Occurrence) -> Result<OccurrenceId, Error> {
        let write = self.lock.write().await;
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
            occurrence.skull,
            occurrence.amount,
            occurrence.millis,
        )
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn delete(&mut self, id: Id) -> Result<OccurrenceId, Error> {
        let write = self.lock.write().await;
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
        .fetch_optional(&*write)
        .await
        .map_err(Into::into)
        .and_then(|d| d.ok_or(Error::NotFound(id)))
    }

    pub async fn last_modified(&self) -> Result<std::time::SystemTime, Error> {
        let read = self.lock.read().await;
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
        .fetch_optional(&*read)
        .await
        .map_err(Into::into)
        .and_then(transient::Time::unpack)
    }
}
