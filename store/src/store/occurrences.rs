use crate::{Error, Result, Store};

pub struct Occurrences<'a> {
    store: &'a Store,
}

impl<'a> Occurrences<'a> {
    pub(super) fn new(store: &'a Store) -> Self {
        Self { store }
    }
}

impl Occurrences<'_> {
    pub async fn list(&self) -> Result<Vec<types::Occurrence>> {
        sqlx::query_as!(
            types::Occurrence,
            r#"
            SELECT
                id AS "id: types::OccurrenceId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32",
                millis AS "millis: chrono::DateTime<chrono::Utc>"
            FROM
                occurrences
            "#
        )
        .fetch_all(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn search(&self, filter: &types::Filter) -> Result<Vec<types::Occurrence>> {
        let mut builder = sqlx::QueryBuilder::new(
            r#"
            SELECT
                id AS "id: types::OccurrenceId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32",
                millis AS "millis: chrono::DateTime<chrono::Utc>"
            FROM
                occurrences
            "#,
        );

        let mut nowhere = false;

        if !filter.skulls.is_empty() {
            if nowhere {
                builder.push(" WHERE skull IN (");
                nowhere = false;
            }
            let mut separated = builder.separated(',');

            for skull in &filter.skulls {
                separated.push_bind(skull);
            }

            separated.push_unseparated(')');
        }

        if let Some(start) = filter.start {
            if nowhere {
                builder.push(" WHERE start >= ");
                nowhere = false;
            } else {
                builder.push(" AND start >= ");
            }
            builder.push_bind(start);
        }

        if let Some(end) = filter.end {
            if nowhere {
                builder.push(" WHERE end <= ");
            } else {
                builder.push(" AND end <= ");
            }
            builder.push_bind(end);
        }

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push(limit);
        }

        builder
            .build_query_as()
            .fetch_all(&self.store.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(
        &self,
        skull: types::SkullId,
        amount: f32,
        millis: chrono::DateTime<chrono::Utc>,
    ) -> Result<types::Occurrence> {
        sqlx::query_as!(
            types::Occurrence,
            r#"
            INSERT INTO occurrences (
                skull,
                amount,
                millis
            ) VALUES (
                $1,
                $2,
                $3
            ) RETURNING
                id AS "id: types::OccurrenceId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32",
                millis AS "millis: chrono::DateTime<chrono::Utc>"
            "#,
            skull,
            amount,
            millis,
        )
        .fetch_one(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(
        &self,
        id: types::OccurrenceId,
        skull: Option<types::SkullId>,
        amount: Option<f32>,
        millis: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<types::Occurrence> {
        match (skull, amount, millis) {
            (Some(skull), Some(amount), Some(millis)) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        skull = $2,
                        amount = $3,
                        millis = $4
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    skull,
                    amount,
                    millis,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, Some(amount), Some(millis)) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        amount = $2,
                        millis = $3
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    amount,
                    millis,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (Some(skull), None, Some(millis)) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        skull = $2,
                        millis = $3
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    skull,
                    millis,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (Some(skull), Some(amount), None) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        skull = $2,
                        amount = $3
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    skull,
                    amount,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (Some(skull), None, None) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        skull = $2
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    skull,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, Some(amount), None) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        amount = $2
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    amount,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, None, Some(millis)) => {
                sqlx::query_as!(
                    types::Occurrence,
                    r#"
                    UPDATE
                        occurrences
                    SET
                        millis = $2
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::OccurrenceId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32",
                        millis AS "millis: chrono::DateTime<chrono::Utc>"
                    "#,
                    id,
                    millis,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, None, None) => {
                return Err(Error::NoChanges);
            }
        }
        .map_err(Into::into)
        .and_then(|r| r.ok_or(Error::NotFound(id.into())))
    }

    pub async fn delete(&self, id: types::OccurrenceId) -> Result {
        sqlx::query!(
            r#"
            DELETE FROM
                occurrences
            WHERE
                id = $1
            RETURNING
                id
            "#,
            id,
        )
        .map(|_| ())
        .fetch_optional(&self.store.pool)
        .await
        .map_err(Into::into)
        .and_then(|r| r.ok_or(Error::NotFound(id.into())))
    }
}
