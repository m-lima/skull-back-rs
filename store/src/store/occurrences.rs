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

    pub async fn search(
        &self,
        skulls: &std::collections::HashSet<types::SkullId>,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<types::Occurrence>> {
        let mut builder = sqlx::QueryBuilder::new(
            r#"
            SELECT
                id,
                skull,
                amount,
                millis
            FROM
                occurrences
            "#,
        );

        let mut nowhere = true;

        if !skulls.is_empty() {
            if nowhere {
                builder.push(" WHERE skull IN (");
                nowhere = false;
            }
            let mut separated = builder.separated(',');

            for skull in skulls {
                separated.push_bind(skull);
            }

            separated.push_unseparated(')');
        }

        if let Some(start) = start {
            if nowhere {
                builder.push(" WHERE millis >= ");
                nowhere = false;
            } else {
                builder.push(" AND millis >= ");
            }
            builder.push_bind(start);
        }

        if let Some(end) = end {
            if nowhere {
                builder.push(" WHERE millis <= ");
            } else {
                builder.push(" AND millis <= ");
            }
            builder.push_bind(end);
        }

        if let Some(limit) = limit {
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
        if amount < 0.0 {
            return Err(Error::InvalidParameter("amount"));
        }

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

    // allow(clippy::too_many_lines): So that we can have static type checking
    #[allow(clippy::too_many_lines)]
    pub async fn update(
        &self,
        id: types::OccurrenceId,
        skull: Option<types::SkullId>,
        amount: Option<f32>,
        millis: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<types::Occurrence> {
        if let Some(amount) = amount {
            if amount < 0.0 {
                return Err(Error::InvalidParameter("amount"));
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    async fn skulled_store() -> (Store, types::Skull) {
        let store = Store::in_memory(1).await.unwrap();

        let skull = store
            .skulls()
            .create("one", 1, "icon1", 1.0, None)
            .await
            .unwrap();

        (store, skull)
    }

    fn chrono(value: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(value, 0).unwrap()
    }

    async fn prepare_search() -> (
        Store,
        (types::SkullId, types::SkullId),
        [types::OccurrenceId; 6],
    ) {
        let (store, skull) = skulled_store().await;
        let skull_id = skull.id;
        let other_id = store
            .skulls()
            .create("two", 2, "two", 2.0, None)
            .await
            .unwrap()
            .id;

        let occurrences = store.occurrences();

        let one = occurrences
            .create(skull_id, 1.0, chrono(1))
            .await
            .unwrap()
            .id;
        let two = occurrences
            .create(skull_id, 2.0, chrono(2))
            .await
            .unwrap()
            .id;
        let three = occurrences
            .create(skull_id, 3.0, chrono(3))
            .await
            .unwrap()
            .id;
        let four = occurrences
            .create(skull_id, 4.0, chrono(4))
            .await
            .unwrap()
            .id;
        let five = occurrences
            .create(other_id, 3.0, chrono(3))
            .await
            .unwrap()
            .id;
        let six = occurrences
            .create(other_id, 4.0, chrono(4))
            .await
            .unwrap()
            .id;

        (
            store,
            (skull_id, other_id),
            [one, two, three, four, five, six],
        )
    }

    #[tokio::test]
    async fn list() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let one = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let two = occurrences.create(skull.id, 2.0, chrono(2)).await.unwrap();

        let occurrences = occurrences.list().await.unwrap();
        assert_eq!(occurrences, vec![one, two]);
    }

    #[tokio::test]
    async fn list_empty() {
        let store = Store::in_memory(1).await.unwrap();

        let occurrences = store.occurrences();
        let occurrences = occurrences.list().await.unwrap();
        assert_eq!(occurrences, Vec::new());
    }

    #[tokio::test]
    async fn search_no_filters() {
        let (store, _, ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(&std::collections::HashSet::new(), None, None, None)
            .await
            .unwrap();
        assert_eq!(occurrences.iter().map(|o| o.id).collect::<Vec<_>>(), ids);
    }

    #[tokio::test]
    async fn search_all_skulls() {
        let (store, (skull_one, skull_two), ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::from([skull_one, skull_two]),
                None,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(occurrences.iter().map(|o| o.id).collect::<Vec<_>>(), ids);
    }

    #[tokio::test]
    async fn search_just_skulls() {
        let (store, (_, skull_two), ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::from([skull_two]),
                None,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            ids[4..]
        );
    }

    #[tokio::test]
    async fn search_just_start() {
        let (store, _, ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::new(),
                Some(chrono(3)),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            ids[2..]
        );
    }

    #[tokio::test]
    async fn search_just_end() {
        let (store, _, ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::new(),
                None,
                Some(chrono(2)),
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            ids[..2]
        );
    }

    #[tokio::test]
    async fn search_start_and_end() {
        let (store, _, ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::new(),
                Some(chrono(3)),
                Some(chrono(3)),
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            [ids[2], ids[4]]
        );
    }

    #[tokio::test]
    async fn search_just_limit() {
        let (store, _, ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(&std::collections::HashSet::new(), None, None, Some(3))
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            ids[..3]
        );
    }

    #[tokio::test]
    async fn search_all_filters() {
        let (store, (skull_one, _), ids) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                &std::collections::HashSet::from([skull_one]),
                Some(chrono(3)),
                Some(chrono(4)),
                Some(1),
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences.iter().map(|o| o.id).collect::<Vec<_>>(),
            [ids[2]]
        );
    }

    #[tokio::test]
    async fn create() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, skull.id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, chrono(1));
    }

    #[tokio::test]
    async fn create_err_no_skull() {
        let (store, skull) = skulled_store().await;
        store.skulls().delete(skull.id).await.unwrap();

        let occurrences = store.occurrences();

        let err = occurrences
            .create(skull.id, 1.0, chrono(1))
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::ForeignKey.to_string());
    }

    #[tokio::test]
    async fn create_err_amount_negative() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();

        let err = occurrences
            .create(skull.id, -1.0, chrono(1))
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("amount").to_string()
        );
    }

    #[tokio::test]
    async fn update() {
        let (store, skull) = skulled_store().await;
        let other_id = store
            .skulls()
            .create("two", 2, "two", 2.0, None)
            .await
            .unwrap()
            .id;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let occurrence = occurrences
            .update(occurrence.id, Some(other_id), Some(2.0), Some(chrono(2)))
            .await
            .unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 2.0.to_string());
        assert_eq!(occurrence.millis, chrono(2));
    }

    #[tokio::test]
    async fn update_same_values() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let occurrence = occurrences
            .update(occurrence.id, Some(skull.id), Some(1.0), Some(chrono(1)))
            .await
            .unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, skull.id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, chrono(1));
    }

    #[tokio::test]
    async fn update_parts() {
        let (store, skull) = skulled_store().await;
        let other_id = store
            .skulls()
            .create("two", 2, "two", 2.0, None)
            .await
            .unwrap()
            .id;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();

        let occurrence = occurrences
            .update(occurrence.id, Some(other_id), None, None)
            .await
            .unwrap();
        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, chrono(1));

        let occurrence = occurrences
            .update(occurrence.id, None, Some(2.0), Some(chrono(2)))
            .await
            .unwrap();
        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 2.0.to_string());
        assert_eq!(occurrence.millis, chrono(2));
    }

    #[tokio::test]
    async fn update_err_no_changes() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let err = occurrences
            .update(occurrence.id, None, None, None)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), Error::NoChanges.to_string());
    }

    #[tokio::test]
    async fn update_err_not_found() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        occurrences.delete(occurrence.id).await.unwrap();
        let err = occurrences
            .update(occurrence.id, None, Some(2.0), None)
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            Error::NotFound(occurrence.id.into()).to_string()
        );
    }

    #[tokio::test]
    async fn update_err_no_skull() {
        let (store, skull) = skulled_store().await;
        let other_id = store
            .skulls()
            .create("two", 2, "two", 2.0, None)
            .await
            .unwrap()
            .id;
        store.skulls().delete(other_id).await.unwrap();

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let err = occurrences
            .update(occurrence.id, Some(other_id), None, None)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), Error::ForeignKey.to_string());
    }

    #[tokio::test]
    async fn update_err_amount_negative() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        let err = occurrences
            .update(occurrence.id, None, Some(-1.0), None)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("amount").to_string()
        );
    }

    #[tokio::test]
    async fn delete() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        occurrences.delete(occurrence.id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_err_not_found() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = occurrences.create(skull.id, 1.0, chrono(1)).await.unwrap();
        occurrences.delete(occurrence.id).await.unwrap();

        let err = occurrences.delete(occurrence.id).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::NotFound(occurrence.id.into()).to_string()
        );
    }
}
