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
    #[tracing::instrument(skip(self), err)]
    pub async fn list(&self) -> Result<Vec<types::Occurrence>> {
        sqlx::query_as!(
            types::Occurrence,
            r#"
            SELECT
                id AS "id: types::OccurrenceId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32",
                millis AS "millis: types::Millis"
            FROM
                occurrences
            ORDER BY
                millis DESC,
                skull DESC
            "#
        )
        .fetch_all(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn search(
        &self,
        skulls: Option<&std::collections::HashSet<types::SkullId>>,
        start: Option<types::Millis>,
        end: Option<types::Millis>,
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

        if let Some(skulls) = skulls {
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

        builder.push(" ORDER BY millis DESC, skull DESC ");

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

    #[tracing::instrument(skip(self), err)]
    pub async fn create<
        I: IntoIterator<Item = (types::SkullId, f32, types::Millis)> + std::fmt::Debug,
    >(
        &self,
        items: I,
    ) -> Result<Vec<types::Occurrence>> {
        let mut tx = self.store.pool.begin().await?;
        let mut occurrences = Vec::new();

        for (skull, amount, millis) in items {
            if amount <= 0.0 {
                return Err(Error::InvalidParameter("amount"));
            }

            let occurrence = sqlx::query_as!(
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
                    millis AS "millis: types::Millis"
                "#,
                skull,
                amount,
                millis,
            )
            .fetch_one(tx.as_mut())
            .await?;

            occurrences.push(occurrence);
        }

        if occurrences.is_empty() {
            return Err(Error::NoChanges);
        }

        tx.commit().await?;
        Ok(occurrences)
    }

    // allow(clippy::too_many_lines): So that we can have static type checking
    #[allow(clippy::too_many_lines)]
    #[tracing::instrument(skip(self), err)]
    pub async fn update(
        &self,
        id: types::OccurrenceId,
        skull: Option<types::SkullId>,
        amount: Option<f32>,
        millis: Option<types::Millis>,
    ) -> Result<types::Occurrence> {
        if let Some(amount) = amount {
            if amount <= 0.0 {
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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
                        millis AS "millis: types::Millis"
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

    #[tracing::instrument(skip(self), err)]
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

    fn millis(value: i64) -> types::Millis {
        types::Millis::from(value)
    }

    async fn prepare_search() -> (
        Store,
        (types::SkullId, types::SkullId),
        [types::Occurrence; 6],
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

        let mut new_occurrences = [
            (other_id, 4.0, millis(4)),
            (skull_id, 4.0, millis(4)),
            (other_id, 3.0, millis(3)),
            (skull_id, 3.0, millis(3)),
            (skull_id, 2.0, millis(2)),
            (skull_id, 1.0, millis(1)),
        ];
        sort(&mut new_occurrences);

        let occurrences = occurrences
            .create(new_occurrences)
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        (store, (skull_id, other_id), occurrences.try_into().unwrap())
    }

    async fn create_plain(store: &Store, skull: &types::Skull) -> types::Occurrence {
        store
            .occurrences()
            .create([(skull.id, 1.0, millis(1))])
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    trait Sortable {
        fn millis(&self) -> &types::Millis;
        fn skull(&self) -> &types::SkullId;
    }

    impl Sortable for types::Occurrence {
        fn millis(&self) -> &types::Millis {
            &self.millis
        }

        fn skull(&self) -> &types::SkullId {
            &self.skull
        }
    }

    impl Sortable for (types::SkullId, f32, types::Millis) {
        fn millis(&self) -> &types::Millis {
            &self.2
        }

        fn skull(&self) -> &types::SkullId {
            &self.0
        }
    }

    fn sort<S: Sortable>(occurrences: &mut [S]) {
        occurrences.sort_unstable_by(|a, b| match b.millis().cmp(a.millis()) {
            std::cmp::Ordering::Equal => b.skull().cmp(a.skull()),
            c => c,
        });
    }

    fn filter(
        occurrences: [types::Occurrence; 6],
        filter: impl Fn(&types::Occurrence) -> bool,
    ) -> Vec<types::Occurrence> {
        occurrences.into_iter().filter(filter).collect()
    }

    #[tokio::test]
    async fn list() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let mut result = occurrences
            .create([(skull.id, 1.0, millis(1)), (skull.id, 2.0, millis(2))])
            .await
            .unwrap();
        sort(&mut result);

        let occurrences = occurrences.list().await.unwrap();
        assert_eq!(occurrences, result);
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
        let (store, _, news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(None, None, None, None)
            .await
            .unwrap();
        assert_eq!(occurrences, news);
    }

    #[tokio::test]
    async fn search_all_skulls() {
        let (store, (skull_one, skull_two), news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                Some(&std::collections::HashSet::from([skull_one, skull_two])),
                None,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(occurrences, news);
    }

    #[tokio::test]
    async fn search_no_skulls() {
        let (store, _, _) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(Some(&std::collections::HashSet::new()), None, None, None)
            .await
            .unwrap();
        assert_eq!(occurrences.iter().map(|o| o.id).collect::<Vec<_>>(), []);
    }

    #[tokio::test]
    async fn search_just_skulls() {
        let (store, (_, skull_two), news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                Some(&std::collections::HashSet::from([skull_two])),
                None,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(occurrences, filter(news, |o| o.skull == skull_two));
    }

    #[tokio::test]
    async fn search_just_start() {
        let (store, _, news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(None, Some(millis(3)), None, None)
            .await
            .unwrap();
        assert_eq!(occurrences, filter(news, |o| o.millis >= millis(3)));
    }

    #[tokio::test]
    async fn search_just_end() {
        let (store, _, news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(None, None, Some(millis(2)), None)
            .await
            .unwrap();
        assert_eq!(occurrences, filter(news, |o| o.millis <= millis(2)));
    }

    #[tokio::test]
    async fn search_start_and_end() {
        let (store, _, news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(None, Some(millis(3)), Some(millis(3)), None)
            .await
            .unwrap();
        assert_eq!(
            occurrences,
            filter(news, |o| o.millis >= millis(3) && o.millis <= millis(3))
        );
    }

    #[tokio::test]
    async fn search_just_limit() {
        let (store, _, news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(None, None, None, Some(3))
            .await
            .unwrap();
        assert_eq!(occurrences, news[..3]);
    }

    #[tokio::test]
    async fn search_all_filters() {
        let (store, (skull_one, _), news) = prepare_search().await;

        let occurrences = store
            .occurrences()
            .search(
                Some(&std::collections::HashSet::from([skull_one])),
                Some(millis(3)),
                Some(millis(4)),
                Some(1),
            )
            .await
            .unwrap();
        assert_eq!(
            occurrences,
            filter(news, |o| o.skull == skull_one
                && o.millis >= millis(3)
                && o.millis <= millis(4))[..1]
        );
    }

    #[tokio::test]
    async fn create() {
        let (store, skull) = skulled_store().await;

        let occurrence = create_plain(&store, &skull).await;

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, skull.id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, millis(1));
    }

    #[tokio::test]
    async fn create_err_no_skull() {
        let (store, skull) = skulled_store().await;
        store.skulls().delete(skull.id).await.unwrap();

        let occurrences = store.occurrences();

        let err = occurrences
            .create([(skull.id, 1.0, millis(1))])
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::ForeignKey.to_string());
    }

    #[tokio::test]
    async fn create_err_amount_negative() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();

        let err = occurrences
            .create([(skull.id, -1.0, millis(1))])
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("amount").to_string()
        );
    }

    #[tokio::test]
    async fn create_err_amount_zero() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();

        let err = occurrences
            .create([(skull.id, 0.0, millis(1))])
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
        let occurrence = create_plain(&store, &skull).await;
        let occurrence = occurrences
            .update(occurrence.id, Some(other_id), Some(2.0), Some(millis(2)))
            .await
            .unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 2.0.to_string());
        assert_eq!(occurrence.millis, millis(2));
    }

    #[tokio::test]
    async fn update_same_values() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = create_plain(&store, &skull).await;
        let occurrence = occurrences
            .update(occurrence.id, Some(skull.id), Some(1.0), Some(millis(1)))
            .await
            .unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, skull.id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, millis(1));
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
        let occurrence = create_plain(&store, &skull).await;
        let occurrence = occurrences
            .update(occurrence.id, Some(other_id), None, None)
            .await
            .unwrap();

        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 1.0.to_string());
        assert_eq!(occurrence.millis, millis(1));

        let occurrence = occurrences
            .update(occurrence.id, None, Some(2.0), Some(millis(2)))
            .await
            .unwrap();
        assert_eq!(types::Id::from(occurrence.id), 1);
        assert_eq!(occurrence.skull, other_id);
        assert_eq!(occurrence.amount.to_string(), 2.0.to_string());
        assert_eq!(occurrence.millis, millis(2));
    }

    #[tokio::test]
    async fn update_err_no_changes() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = create_plain(&store, &skull).await;
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
        let occurrence = create_plain(&store, &skull).await;
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
        let occurrence = create_plain(&store, &skull).await;
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
        let occurrence = create_plain(&store, &skull).await;
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
    async fn update_err_amount_zero() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = create_plain(&store, &skull).await;
        let err = occurrences
            .update(occurrence.id, None, Some(0.0), None)
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
        let occurrence = create_plain(&store, &skull).await;
        occurrences.delete(occurrence.id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_err_not_found() {
        let (store, skull) = skulled_store().await;

        let occurrences = store.occurrences();
        let occurrence = create_plain(&store, &skull).await;
        occurrences.delete(occurrence.id).await.unwrap();

        let err = occurrences.delete(occurrence.id).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::NotFound(occurrence.id.into()).to_string()
        );
    }
}
