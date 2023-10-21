use crate::{Error, Result, Store};

pub struct Skulls<'a> {
    store: &'a Store,
}

impl<'a> Skulls<'a> {
    pub(super) fn new(store: &'a Store) -> Self {
        Self { store }
    }
}

impl Skulls<'_> {
    pub async fn list(&self) -> Result<Vec<types::Skull>> {
        sqlx::query_as!(
            types::Skull,
            r#"
            SELECT
                "id" AS "id: types::SkullId",
                "name",
                "color" AS "color: u32",
                "icon",
                "unit_price" AS "unit_price: f32",
                "limit" as "limit: f32"
            FROM
                skulls
            "#
        )
        .fetch_all(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create<
        Name: AsRef<str> + std::fmt::Debug,
        Icon: std::ops::Deref<Target = str> + std::fmt::Debug,
    >(
        &self,
        name: Name,
        color: u32,
        icon: Icon,
        unit_price: f32,
        limit: Option<f32>,
    ) -> Result<types::Skull> {
        let name = super::check_non_empty(name.as_ref(), "name")?;
        let icon = super::check_non_empty(icon.as_ref(), "icon")?;
        if unit_price < 0.0 {
            return Err(Error::InvalidParameter("unit_price"));
        }

        sqlx::query_as!(
            types::Skull,
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
                "id" AS "id: types::SkullId",
                "name",
                "color" AS "color: u32",
                "icon",
                "unit_price" AS "unit_price: f32",
                "limit" as "limit: f32"
            "#,
            name,
            color,
            icon,
            unit_price,
            limit,
        )
        .fetch_one(&self.store.pool)
        .await
        .map_err(Into::into)
    }

    // allow(clippy::option_option): This is necessary to convey a change into None
    #[allow(clippy::option_option)]
    pub async fn update<
        Name: AsRef<str> + std::fmt::Debug,
        Icon: std::ops::Deref<Target = str> + std::fmt::Debug,
    >(
        &self,
        id: types::SkullId,
        name: Option<Name>,
        color: Option<u32>,
        icon: Option<Icon>,
        unit_price: Option<f32>,
        limit: Option<Option<f32>>,
    ) -> Result<types::Skull> {
        let mut has_fields = false;
        let mut builder = sqlx::QueryBuilder::new("UPDATE skulls SET ");
        let mut fields = builder.separated(',');

        macro_rules! push_field {
            ($name: ident) => {
                push_field!($name, $name)
            };
            ($name: ident, $push: expr) => {
                if let Some($name) = $name {
                    let $name = $push;
                    fields
                        .push(concat!("\"", stringify!($name), "\"", " = "))
                        .push_bind_unseparated($name);
                    has_fields = true;
                }
            };
        }

        push_field!(
            name,
            String::from(super::check_non_empty(name.as_ref(), "name")?)
        );
        push_field!(color);
        push_field!(
            icon,
            String::from(super::check_non_empty(icon.as_ref(), "icon")?)
        );
        push_field!(
            unit_price,
            if unit_price < 0.0 {
                return Err(Error::InvalidParameter("unit_price"));
            } else {
                unit_price
            }
        );
        push_field!(limit);

        if has_fields {
            builder
                .push(" WHERE id = ")
                .push_bind(id)
                .push(
                    r#"
                    RETURNING
                        "id",
                        "name",
                        "color",
                        "icon",
                        "unit_price",
                        "limit"
                    "#,
                )
                .build_query_as::<types::Skull>()
                .fetch_optional(&self.store.pool)
                .await
                .map_err(Into::into)
                .and_then(|r| r.ok_or(Error::NotFound(id.into())))
        } else {
            Err(Error::NoChanges)
        }
    }

    pub async fn delete(&self, id: types::SkullId) -> Result {
        sqlx::query!(
            r#"
            DELETE FROM
                skulls
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
    async fn list() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let one = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        let two = skulls
            .create("two", 2, "icon2", 2.0, Some(2.0))
            .await
            .unwrap();

        let skulls = skulls.list().await.unwrap();
        assert_eq!(skulls, vec![one, two]);
    }

    #[tokio::test]
    async fn list_empty() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skulls = skulls.list().await.unwrap();
        assert_eq!(skulls, Vec::new());
    }

    #[tokio::test]
    async fn create() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "one");
        assert_eq!(skull.color, 1);
        assert_eq!(skull.icon, "icon1");
        assert_eq!(skull.unit_price.to_string(), 1.0.to_string());
        assert_eq!(skull.limit, None);
    }

    #[tokio::test]
    async fn create_err_name_blank() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();

        let err = skulls.create("", 1, "icon1", 1.0, None).await.unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("name").to_string());

        let err = skulls
            .create(" 	 ", 1, "icon1", 1.0, None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("name").to_string());
    }

    #[tokio::test]
    async fn create_err_name_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .create("one", 2, "icon2", 2.0, None)
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn create_err_color_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .create("two", 1, "icon2", 2.0, None)
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn create_err_icon_blank() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();

        let err = skulls.create("one", 1, "", 1.0, None).await.unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("icon").to_string());

        let err = skulls.create("one", 1, " 	 ", 1.0, None).await.unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("icon").to_string());
    }

    #[tokio::test]
    async fn create_err_icon_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .create("two", 2, "icon1", 2.0, None)
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn create_err_unit_price_negative() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let err = skulls
            .create("one", 1, "icon1", -1.0, None)
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("unit_price").to_string()
        );
    }

    #[tokio::test]
    async fn update() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        let skull = skulls
            .update(
                skull.id,
                Some("two"),
                Some(2),
                Some("icon2"),
                Some(2.0),
                Some(Some(2.0)),
            )
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "two");
        assert_eq!(skull.color, 2);
        assert_eq!(skull.icon, "icon2");
        assert_eq!(skull.unit_price.to_string(), 2.0.to_string());
        assert_eq!(skull.limit, Some(2.0));
    }

    #[tokio::test]
    async fn update_same_values() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        let skull = skulls
            .update(
                skull.id,
                Some("one"),
                Some(1),
                Some("icon1"),
                Some(1.0),
                Some(None),
            )
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "one");
        assert_eq!(skull.color, 1);
        assert_eq!(skull.icon, "icon1");
        assert_eq!(skull.unit_price.to_string(), 1.0.to_string());
        assert_eq!(skull.limit, None);
    }

    #[tokio::test]
    async fn update_parts() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let skull = skulls
            .update(skull.id, Some("two"), None, None::<String>, None, None)
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "two");
        assert_eq!(skull.color, 1);
        assert_eq!(skull.icon, "icon1");
        assert_eq!(skull.unit_price.to_string(), 1.0.to_string());
        assert_eq!(skull.limit, None);

        let skull = skulls
            .update(skull.id, None::<String>, Some(2), Some("icon2"), None, None)
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "two");
        assert_eq!(skull.color, 2);
        assert_eq!(skull.icon, "icon2");
        assert_eq!(skull.unit_price.to_string(), 1.0.to_string());
        assert_eq!(skull.limit, None);

        let skull = skulls
            .update(
                skull.id,
                None::<String>,
                None,
                None::<String>,
                Some(2.0),
                Some(Some(1.0)),
            )
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "two");
        assert_eq!(skull.color, 2);
        assert_eq!(skull.icon, "icon2");
        assert_eq!(skull.unit_price.to_string(), 2.0.to_string());
        assert_eq!(skull.limit, Some(1.0));

        let skull = skulls
            .update(
                skull.id,
                None::<String>,
                None,
                None::<String>,
                None,
                Some(None),
            )
            .await
            .unwrap();

        assert_eq!(types::Id::from(skull.id), 1);
        assert_eq!(skull.name, "two");
        assert_eq!(skull.color, 2);
        assert_eq!(skull.icon, "icon2");
        assert_eq!(skull.unit_price.to_string(), 2.0.to_string());
        assert_eq!(skull.limit, None);
    }

    #[tokio::test]
    async fn update_err_no_changes() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        let err = skulls
            .update(skull.id, None::<String>, None, None::<String>, None, None)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), Error::NoChanges.to_string());
    }

    #[tokio::test]
    async fn update_err_not_found() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.delete(skull.id).await.unwrap();
        let err = skulls
            .update(
                skull.id,
                Some("two"),
                Some(2),
                Some("icon2"),
                Some(2.0),
                Some(Some(2.0)),
            )
            .await
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            Error::NotFound(skull.id.into()).to_string()
        );
    }

    #[tokio::test]
    async fn update_err_name_blank() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .update(skull.id, Some(""), None, None::<String>, None, None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("name").to_string());

        let err = skulls
            .update(skull.id, Some(" 	 "), None, None::<String>, None, None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("name").to_string());
    }

    #[tokio::test]
    async fn update_err_name_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.create("two", 2, "icon2", 2.0, None).await.unwrap();

        let err = skulls
            .update(skull.id, Some("two"), None, None::<String>, None, None)
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn update_err_color_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.create("two", 2, "icon2", 2.0, None).await.unwrap();

        let err = skulls
            .update(
                skull.id,
                None::<String>,
                Some(2),
                None::<String>,
                None,
                None,
            )
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn update_err_icon_blank() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .update(skull.id, None::<String>, None, Some(""), None, None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("icon").to_string());

        let err = skulls
            .update(skull.id, None::<String>, None, Some(" 	 "), None, None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Error::InvalidParameter("icon").to_string());
    }

    #[tokio::test]
    async fn update_err_icon_duplicate() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.create("two", 2, "icon2", 2.0, None).await.unwrap();

        let err = skulls
            .update(skull.id, None::<String>, None, Some("icon2"), None, None)
            .await
            .unwrap_err();

        if let Error::DuplicateEntry(_) = err {
        } else {
            panic!("{err}");
        }
    }

    #[tokio::test]
    async fn udpate_err_unit_price_negative() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let err = skulls
            .update(
                skull.id,
                None::<String>,
                None,
                None::<String>,
                Some(-1.0),
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::InvalidParameter("unit_price").to_string()
        );
    }

    #[tokio::test]
    async fn delete() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.delete(skull.id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_cascade() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();

        let quicks = store.quicks();
        let quick = quicks.create(skull.id, 1.0).await.unwrap();
        assert_eq!(quicks.list().await.unwrap(), vec![quick]);

        skulls.delete(skull.id).await.unwrap();
        assert_eq!(quicks.list().await.unwrap(), Vec::new());
    }

    #[tokio::test]
    async fn delete_err_not_found() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        skulls.delete(skull.id).await.unwrap();

        let err = skulls.delete(skull.id).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            Error::NotFound(skull.id.into()).to_string()
        );
    }

    #[tokio::test]
    async fn delete_err_referenced() {
        let store = Store::in_memory(1).await.unwrap();

        let skulls = store.skulls();
        let skull = skulls.create("one", 1, "icon1", 1.0, None).await.unwrap();
        store
            .occurrences()
            .create([(
                skull.id,
                1.0,
                chrono::DateTime::from_timestamp(0, 0).unwrap(),
            )])
            .await
            .unwrap();

        let err = skulls.delete(skull.id).await.unwrap_err();
        if let Error::Constraint(_) = err {
        } else {
            panic!("{err}");
        }
    }
}
