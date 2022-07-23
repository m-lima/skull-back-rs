use crate::check;

use super::{Data, Error, Id, Occurrence, Quick, Selector, Skull, Store, WithId};

pub const USER: &str = "bloink";

pub struct Tester<D: Selector>(std::marker::PhantomData<D>);

impl<D: helper::TesterData> Tester<D> {
    pub async fn selectable(store: &impl Store) {
        helper::populate(store).await;
        assert!(D::select(store, USER).is_ok());
    }

    pub async fn reject_unknown_user(store: &impl Store) {
        helper::populate(store).await;
        let err = D::select(store, "unknown")
            .map(|_| ())
            .unwrap_err()
            .to_string();
        let expected = Error::NoSuchUser(String::from("unknown")).to_string();
        assert_eq!(err, expected);
    }

    pub async fn last_modified(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();
        store.last_modified().await.unwrap();
    }

    pub async fn last_modified_empty(store: &impl Store) {
        let store = D::select(store, USER).unwrap();
        store.last_modified().await.unwrap();
    }

    pub async fn list(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();
        let response = check!(helper::get_unchanged_data(
            store.list(None).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, helper::from_range::<D>(1..=3));
    }

    pub async fn list_limited(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_unchanged_data(
            store.list(Some(0)).await.unwrap(),
            last_modified
        ));
        assert!(response.is_empty());

        let response = check!(helper::get_unchanged_data(
            store.list(Some(1)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, helper::from_range::<D>(3..=3));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(2)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, helper::from_range::<D>(2..=3));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(3)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, helper::from_range::<D>(1..=3));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(4)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, helper::from_range::<D>(1..=3));
    }

    pub async fn list_empty(store: &impl Store) {
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();
        let response = check!(helper::get_unchanged_data(
            store.list(None).await.unwrap(),
            last_modified
        ));
        assert!(response.is_empty());

        for i in 0..5 {
            let response = check!(helper::get_unchanged_data(
                store.list(Some(i)).await.unwrap(),
                last_modified
            ));
            assert!(response.is_empty());
        }
    }

    pub async fn create(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();
        let data = D::new(7);
        let response = check!(helper::get_modified_data(
            store.create(data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 4);

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected.push(D::Id::new(4, data));
        assert_eq!(response, expected);
    }

    pub async fn read(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        for i in 1..=3 {
            let response = check!(helper::get_unchanged_data(
                store.read(u32::from(i)).await.unwrap(),
                last_modified
            ));
            assert_eq!(response, D::with_id(i));
        }
    }

    pub async fn read_not_found(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let err = store.read(0).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `0`");

        let err = store.read(4).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `4`");
    }
}

impl Tester<Skull> {
    pub async fn create_conflict(store: &impl Store) {
        let store = Skull::select(store, USER).unwrap();
        let original = <Skull as helper::TesterData>::new(1);
        let last_modified = store.create(original.clone()).await.unwrap().1;

        let err = store
            .create(original.clone())
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Failed constraint");

        let data = Skull {
            name: original.name.clone(),
            ..helper::TesterData::new(2)
        };
        let err = store.create(data).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Failed constraint");

        let data = Skull {
            color: original.color.clone(),
            ..helper::TesterData::new(2)
        };
        let err = store.create(data).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Failed constraint");

        let data = Skull {
            icon: original.icon.clone(),
            ..helper::TesterData::new(2)
        };
        let err = store.create(data).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Failed constraint");

        let data = Skull {
            unit_price: original.unit_price,
            limit: original.limit,
            ..helper::TesterData::new(2)
        };
        let response = check!(helper::get_modified_data(
            store.create(data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 2);

        let response = store.list(None).await.unwrap().0;
        assert_eq!(
            response,
            [
                <Skull as helper::TesterData>::with_id(1),
                <Skull as Data>::Id::new(2, data)
            ]
        );
    }
}

impl Tester<Quick> {
    pub async fn create_conflict(store: &impl Store) {
        let store = Quick::select(store, USER).unwrap();
        let original = <Quick as helper::TesterData>::new(1);
        let mut expected = Vec::with_capacity(3);

        let last_modified = store.create(original.clone()).await.unwrap().1;
        expected.push(<Quick as Data>::Id::new(1, original.clone()));

        let err = store
            .create(original.clone())
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Failed constraint");

        let data = Quick {
            skull: original.skull,
            ..helper::TesterData::new(2)
        };
        let response = check!(helper::get_modified_data(
            store.create(data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 2);
        expected.push(<Quick as Data>::Id::new(response, data));

        let data = Quick {
            amount: original.amount,
            ..helper::TesterData::new(2)
        };
        let response = check!(helper::get_modified_data(
            store.create(data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 3);
        expected.push(<Quick as Data>::Id::new(response, data));

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, expected);
    }
}

impl Tester<Occurrence> {
    pub async fn create_conflict(_store: &impl Store) {}
}

mod helper {
    use crate::test_util::Assertion;

    use super::{Id, Occurrence, Quick, Selector, Skull, Store, WithId, USER};

    pub trait TesterData: Selector {
        fn new(i: u8) -> Self;

        fn with_id(i: u8) -> Self::Id {
            Self::Id::new(Id::from(i), Self::new(i))
        }
    }

    impl TesterData for Skull {
        fn new(i: u8) -> Self {
            Skull {
                name: format!("name{i}"),
                color: format!("color{i}"),
                icon: format!("icon{i}"),
                unit_price: f32::from(i),
                limit: None,
            }
        }
    }

    impl TesterData for Quick {
        fn new(i: u8) -> Quick {
            Quick {
                skull: u32::from(i) << 1,
                amount: f32::from(i),
            }
        }
    }

    impl TesterData for Occurrence {
        fn new(i: u8) -> Occurrence {
            Occurrence {
                skull: u32::from(i) << 1,
                amount: f32::from(i),
                millis: i64::from(i),
            }
        }
    }

    // allow(clippy::cast_precision_loss): It's only 1, 2, 3..
    #[allow(clippy::cast_precision_loss)]
    pub async fn populate(store: &impl Store) {
        let crud = Skull::select(store, USER).unwrap();
        for i in 1..=3 {
            crud.create(Skull::new(i)).await.unwrap();
        }

        let crud = Quick::select(store, USER).unwrap();
        for i in 1..=3 {
            crud.create(Quick::new(i)).await.unwrap();
        }

        let crud = Occurrence::select(store, USER).unwrap();
        for i in 1..=3 {
            crud.create(Occurrence::new(i)).await.unwrap();
        }
    }

    pub fn get_unchanged_data<T>(
        response: (T, std::time::SystemTime),
        last_modified: std::time::SystemTime,
    ) -> Assertion<T> {
        if response.1 == last_modified {
            Assertion::Ok(response.0)
        } else {
            Assertion::err_ne("Unexpected last_modified change", response.1, last_modified)
        }
    }

    pub fn get_modified_data<T>(
        response: (T, std::time::SystemTime),
        last_modified: std::time::SystemTime,
    ) -> Assertion<T> {
        match response.1.cmp(&last_modified) {
            std::cmp::Ordering::Greater => Assertion::Ok(response.0),
            std::cmp::Ordering::Equal => {
                Assertion::err_eq("Value for last_modified was not incremented", response.1)
            }
            std::cmp::Ordering::Less => {
                Assertion::err_eq("Value for last_modified went back in time", response.1)
            }
        }
    }

    pub fn from_range<D: TesterData>(range: std::ops::RangeInclusive<u8>) -> Vec<D::Id> {
        range.map(D::with_id).collect()
    }
}

#[macro_export]
macro_rules! create_tests {
    ($uut:ident, $instance:expr) => {
        mod crud {
            use super::*;
            $crate::create_tests!(skull, $crate::store::Skull, $uut, $instance);
            $crate::create_tests!(quick, $crate::store::Quick, $uut, $instance);
            $crate::create_tests!(occurrence, $crate::store::Occurrence, $uut, $instance);
        }
    };

    ($name:ident, $data:ty, $uut:ident, $instance:expr) => {
        mod $name {
            use super::*;

            type Tester = $crate::store::test::Tester<$data>;

            #[tokio::test(flavor = "multi_thread")]
            async fn selectable() {
                Tester::selectable(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn reject_unknown_user() {
                Tester::reject_unknown_user(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn last_modified() {
                Tester::last_modified(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn last_modified_empty() {
                Tester::last_modified_empty(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn list() {
                Tester::list(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn list_limited() {
                Tester::list_limited(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn list_empty() {
                Tester::list_empty(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn create() {
                Tester::create(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn create_conflict() {
                Tester::create_conflict(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn read() {
                Tester::read(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn read_not_found() {
                Tester::read_not_found(&$instance).await;
            }

            // #[tokio::test(flavor = "multi_thread")]
            // async fn create_constraint() {
            //     Tester::create_constraint(&$instance).await;
            // }
        }
    };
}
