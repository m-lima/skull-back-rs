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

    #[tokio::test]
    async fn full_check() {
        let store = Store::in_memory(1).await.unwrap();
        store.migrate().await.unwrap();

        let skulls = {
            let skulls = store.skulls();
            let one = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

            assert_eq!(one.name, "one");
            assert_eq!(one.color, 1);
            assert_eq!(one.icon, "icon1");
            assert_eq!(one.unit_price, 1.0);
            assert_eq!(one.limit, None);

            let two = skulls
                .create("two", 2, "icon2", 2.0, Some(2.0))
                .await
                .unwrap();

            assert_eq!(two.name, "two");
            assert_eq!(two.color, 2);
            assert_eq!(two.icon, "icon2");
            assert_eq!(two.unit_price, 2.0);
            assert_eq!(two.limit, Some(2.0));

            let skulls = skulls.list().await.unwrap();
            assert_eq!(skulls, vec![one, two]);
            skulls
        };

        let quicks = store.quicks();
        let one = quicks.create(skulls[0].id, 1.0).await.unwrap();
        assert_eq!(one.skull, skulls[0].id);
        assert_eq!(one.amount, 1.0);

        let two = quicks.create(skulls[0].id, 2.0).await.unwrap();
        assert_eq!(two.skull, skulls[0].id);
        assert_eq!(two.amount, 2.0);

        let three = quicks.create(skulls[1].id, 3.0).await.unwrap();
        assert_eq!(three.skull, skulls[1].id);
        assert_eq!(three.amount, 3.0);

        let four = quicks.create(skulls[1].id, 4.0).await.unwrap();
        assert_eq!(four.skull, skulls[1].id);
        assert_eq!(four.amount, 4.0);

        let quicks = quicks.list().await.unwrap();
        assert_eq!(quicks, vec![one, two, three, four]);

        let occurrences = store.occurrences();
    }
}
