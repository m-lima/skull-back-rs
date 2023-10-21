use crate::{Error, Result, Store};

pub struct Quicks<'a> {
    store: &'a Store,
}

impl<'a> Quicks<'a> {
    pub(super) fn new(store: &'a Store) -> Self {
        Self { store }
    }
}

impl Quicks<'_> {
    pub async fn list(&self) -> Result<Vec<types::Quick>> {
        sqlx::query_as!(
            types::Quick,
            r#"
            SELECT
                id AS "id: types::QuickId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32"
            FROM
                quicks
            "#
        )
        .fetch_all(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(&self, skull: types::SkullId, amount: f32) -> Result<types::Quick> {
        if amount < 0.0 {
            return Err(Error::InvalidParameter("amount"));
        }

        sqlx::query_as!(
            types::Quick,
            r#"
            INSERT INTO quicks (
                skull,
                amount
            ) VALUES (
                $1,
                $2
            ) RETURNING
                id AS "id: types::QuickId",
                skull AS "skull: types::SkullId",
                amount AS "amount: f32"
            "#,
            skull,
            amount,
        )
        .fetch_one(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(
        &self,
        id: types::QuickId,
        skull: Option<types::SkullId>,
        amount: Option<f32>,
    ) -> Result<types::Quick> {
        if let Some(amount) = amount {
            if amount < 0.0 {
                return Err(Error::InvalidParameter("amount"));
            }
        }

        match (skull, amount) {
            (Some(skull), Some(amount)) => {
                sqlx::query_as!(
                    types::Quick,
                    r#"
                    UPDATE
                        quicks
                    SET
                        skull = $2,
                        amount = $3
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::QuickId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32"
                    "#,
                    id,
                    skull,
                    amount,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (Some(skull), None) => {
                sqlx::query_as!(
                    types::Quick,
                    r#"
                    UPDATE
                        quicks
                    SET
                        skull = $2
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::QuickId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32"
                    "#,
                    id,
                    skull,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, Some(amount)) => {
                sqlx::query_as!(
                    types::Quick,
                    r#"
                    UPDATE
                        quicks
                    SET
                        amount = $2
                    WHERE
                        id = $1
                    RETURNING
                        id AS "id!: types::QuickId",
                        skull AS "skull: types::SkullId",
                        amount AS "amount: f32"
                    "#,
                    id,
                    amount,
                )
                .fetch_optional(&self.store.pool)
                .await
            }
            (None, None) => {
                return Err(Error::NoChanges);
            }
        }
        .map_err(Into::into)
        .and_then(|r| r.ok_or(Error::NotFound(id.into())))
    }

    pub async fn delete(&self, id: types::QuickId) -> Result {
        sqlx::query!(
            r#"
            DELETE FROM
                quicks
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

    #[tokio::test]
    async fn list() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let one = quicks.create(skull.id, 1.0).await.unwrap();
        let two = quicks.create(skull.id, 2.0).await.unwrap();

        let quicks = quicks.list().await.unwrap();
        assert_eq!(quicks, vec![one, two]);
    }

    #[tokio::test]
    async fn list_empty() {
        let store = Store::in_memory(1).await.unwrap();

        let quicks = store.quicks();
        let quicks = quicks.list().await.unwrap();
        assert_eq!(quicks, Vec::new());
    }

    #[tokio::test]
    async fn create() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();

        assert_eq!(types::Id::from(quick.id), 1);
        assert_eq!(quick.skull, skull.id);
        assert_eq!(quick.amount.to_string(), 1.0.to_string());
    }

    #[tokio::test]
    async fn create_err_no_skull() {
        let (store, skull) = skulled_store().await;
        store.skulls().delete(skull.id).await.unwrap();

        let quicks = store.quicks();

        let err = quicks.create(skull.id, 1.0).await.unwrap_err();
        assert_eq!(err.to_string(), Error::ForeignKey.to_string());
    }

    #[tokio::test]
    async fn create_err_amount_negative() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();

        let err = quicks.create(skull.id, -1.0).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("amount").to_string()
        );
    }

    #[tokio::test]
    async fn create_err_duplicate() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        quicks.create(skull.id, 1.0).await.unwrap();

        let err = quicks.create(skull.id, 1.0).await.unwrap_err();
        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
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

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        let quick = quicks
            .update(quick.id, Some(other_id), Some(2.0))
            .await
            .unwrap();

        assert_eq!(types::Id::from(quick.id), 1);
        assert_eq!(quick.skull, other_id);
        assert_eq!(quick.amount.to_string(), 2.0.to_string());
    }

    #[tokio::test]
    async fn update_same_values() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        let quick = quicks
            .update(quick.id, Some(skull.id), Some(1.0))
            .await
            .unwrap();

        assert_eq!(types::Id::from(quick.id), 1);
        assert_eq!(quick.skull, skull.id);
        assert_eq!(quick.amount.to_string(), 1.0.to_string());
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

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();

        let quick = quicks.update(quick.id, Some(other_id), None).await.unwrap();
        assert_eq!(types::Id::from(quick.id), 1);
        assert_eq!(quick.skull, other_id);
        assert_eq!(quick.amount.to_string(), 1.0.to_string());

        let quick = quicks.update(quick.id, None, Some(2.0)).await.unwrap();
        assert_eq!(types::Id::from(quick.id), 1);
        assert_eq!(quick.skull, other_id);
        assert_eq!(quick.amount.to_string(), 2.0.to_string());
    }

    #[tokio::test]
    async fn update_err_no_changes() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        let err = quicks.update(quick.id, None, None).await.unwrap_err();

        assert_eq!(err.to_string(), Error::NoChanges.to_string());
    }

    #[tokio::test]
    async fn update_err_not_found() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        quicks.delete(quick.id).await.unwrap();
        let err = quicks.update(quick.id, None, Some(2.0)).await.unwrap_err();

        assert_eq!(
            err.to_string(),
            Error::NotFound(quick.id.into()).to_string()
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

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        let err = quicks
            .update(quick.id, Some(other_id), None)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), Error::ForeignKey.to_string());
    }

    #[tokio::test]
    async fn update_err_amount_negative() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        let err = quicks.update(quick.id, None, Some(-1.0)).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("amount").to_string()
        );
    }

    #[tokio::test]
    async fn update_err_duplicate() {
        let (store, skull) = skulled_store().await;
        let other_id = store
            .skulls()
            .create("two", 2, "two", 2.0, None)
            .await
            .unwrap()
            .id;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        quicks.create(skull.id, 2.0).await.unwrap();
        quicks.create(other_id, 1.0).await.unwrap();

        let err = quicks.update(quick.id, None, Some(2.0)).await.unwrap_err();
        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }

        let err = quicks
            .update(quick.id, Some(other_id), None)
            .await
            .unwrap_err();
        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn delete() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        quicks.delete(quick.id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_err_not_found() {
        let (store, skull) = skulled_store().await;

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        quicks.delete(quick.id).await.unwrap();

        let err = quicks.delete(quick.id).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::NotFound(quick.id.into()).to_string()
        );
    }
}
