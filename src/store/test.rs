#[macro_export]
macro_rules! create_tests {
    ($uut:ident, $instance:expr) => {
        mod crud {
            use super::*;

            mod global {
                use super::*;

                #[tokio::test(flavor = "multi_thread")]
                async fn last_modified_does_not_leak() {
                    $crate::store::test::last_modified_does_not_leak(&$instance).await;
                }

                #[tokio::test(flavor = "multi_thread")]
                async fn delete_cascade() {
                    $crate::store::test::delete_cascade(&$instance).await;
                }

                #[tokio::test(flavor = "multi_thread")]
                async fn delete_reject() {
                    $crate::store::test::delete_reject(&$instance).await;
                }

                #[tokio::test(flavor = "multi_thread")]
                async fn multiple_handles() {
                    $crate::store::test::multiple_handles($instance).await;
                }
            }

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
            async fn create_constraint() {
                Tester::create_constraint(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn create_monotonic() {
                Tester::create_monotonic(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn read() {
                Tester::read(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn read_sparse() {
                Tester::read_sparse(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn read_not_found() {
                Tester::read_not_found(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn update() {
                Tester::update(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn update_no_changes() {
                Tester::update_no_changes(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn update_not_found() {
                Tester::update_not_found(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn update_conflict() {
                Tester::update_conflict(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn update_constraint() {
                Tester::update_constraint(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn delete() {
                Tester::delete(&$instance).await;
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn delete_not_found() {
                Tester::delete_not_found(&$instance).await;
            }
        }
    };
}
use crate::check;

use super::{Error, Id, Occurrence, Quick, Selector, Skull, Store, WithId};

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

    pub async fn create_conflict(store: &impl Store) {
        let store = D::select(store, USER).unwrap();

        let initial = D::new(1);
        let last_modified = store.create(initial.clone()).await.unwrap().1;
        let mut expected = vec![D::Id::new(1, initial.clone())];

        for conflicting in initial.make_conflicts() {
            let err = store.create(conflicting).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");
        }

        for (id, non_conflicting) in initial.make_non_conflicts().into_iter().enumerate() {
            let response = check!(helper::get_modified_data(
                store.create(non_conflicting.clone()).await.unwrap(),
                last_modified
            ));
            let expected_id = Id::try_from(id + 2).unwrap();
            assert_eq!(response, expected_id);
            expected.push(D::Id::new(expected_id, non_conflicting));
        }

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, expected);
    }

    pub async fn create_constraint(store: &impl Store) {
        if let Some(unconstraint) = D::make_unconstraint() {
            helper::populate(store).await;
            let store = D::select(store, USER).unwrap();

            let last_modified = store.last_modified().await.unwrap();

            let err = store.create(unconstraint).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");

            let response = store.list(None).await.unwrap().0;
            assert_eq!(response, helper::from_range::<D>(1..=3));
        }
    }

    pub async fn create_monotonic(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        for i in 4..=20 {
            store.create(D::new(i)).await.unwrap();
        }

        for i in 1..=20 {
            if i % 3 == 0 || i % 4 == 0 {
                store.delete(i).await.unwrap();
            }
        }

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_modified_data(
            store.create(D::new(3)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 21);
    }

    pub async fn read(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        for i in 1..=3 {
            let response = check!(helper::get_unchanged_data(
                store.read(Id::from(i)).await.unwrap(),
                last_modified
            ));
            assert_eq!(response, D::with_id(i));
        }
    }

    pub async fn read_sparse(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        for i in 4..=20 {
            store.create(D::new(i)).await.unwrap();
        }

        for i in 1..=20 {
            if i % 3 == 0 || i % 4 == 0 {
                store.delete(i).await.unwrap();
            }
        }

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_unchanged_data(
            store.read(11).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::with_id(11));
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

        let err = store.read(Id::MAX).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, format!("Entry not found for id `{}`", Id::MAX));
    }

    pub async fn update(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let data = D::new(7);
        let response = check!(helper::get_modified_data(
            store.update(2, data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::with_id(2));

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected[1] = D::Id::new(2, data);
        assert_eq!(response, expected);
    }

    pub async fn update_no_changes(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_unchanged_data(
            store.update(2, D::new(2)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::with_id(2));

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, helper::from_range::<D>(1..=3));
    }

    pub async fn update_not_found(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let data = D::new(7);
        let err = store.update(0, data.clone()).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `0`");

        let err = store.update(4, data.clone()).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `4`");

        let err = store
            .update(Id::MAX, data.clone())
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, format!("Entry not found for id `{}`", Id::MAX));

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, helper::from_range::<D>(1..=3));
    }

    pub async fn update_conflict(store: &impl Store) {
        let store = D::select(store, USER).unwrap();

        let initial = D::new(1);
        store.create(initial.clone()).await.unwrap();

        let mut last_modified = store.create(D::new(2)).await.unwrap().1;
        let mut expected = helper::from_range::<D>(1..=2);

        for (i, _) in initial.make_non_conflicts().iter().enumerate() {
            last_modified = store
                .create(D::new(u8::try_from(i + 3).unwrap()))
                .await
                .unwrap()
                .1;
        }

        for conflicting in initial.make_conflicts() {
            let err = store.update(2, conflicting).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");
        }

        for (id, non_conflicting) in initial.make_non_conflicts().into_iter().enumerate() {
            let id = Id::try_from(id + 3).unwrap();
            let response = check!(helper::get_modified_data(
                store.update(id, non_conflicting.clone()).await.unwrap(),
                last_modified
            ));
            assert_eq!(response, D::new(u8::try_from(id).unwrap()));
            expected.push(D::Id::new(id, non_conflicting));
        }

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, expected);
    }

    pub async fn update_constraint(store: &impl Store) {
        if let Some(unconstraint) = D::make_unconstraint() {
            helper::populate(store).await;
            let store = D::select(store, USER).unwrap();

            let last_modified = store.last_modified().await.unwrap();

            let err = store.update(2, unconstraint).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");

            let response = store.list(None).await.unwrap().0;
            assert_eq!(response, helper::from_range::<D>(1..=3));
        }
    }

    pub async fn delete(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_modified_data(
            store.delete(2).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::with_id(2));

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected.remove(1);
        assert_eq!(response, expected);
    }

    pub async fn delete_not_found(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let err = store.delete(0).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `0`");

        let err = store.delete(4).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, "Entry not found for id `4`");

        let err = store.delete(Id::MAX).await.unwrap_err().to_string();
        assert_eq!(store.last_modified().await.unwrap(), last_modified);
        assert_eq!(err, format!("Entry not found for id `{}`", Id::MAX));

        let response = store.list(None).await.unwrap().0;
        assert_eq!(response, helper::from_range::<D>(1..=3));
    }
}

pub async fn last_modified_does_not_leak(store: &impl Store) {
    helper::populate(store).await;

    let skull_last_modified = Skull::select(store, USER)
        .unwrap()
        .last_modified()
        .await
        .unwrap();
    let quick_last_modified = Quick::select(store, USER)
        .unwrap()
        .last_modified()
        .await
        .unwrap();
    let occurrence_last_modified = Occurrence::select(store, USER)
        .unwrap()
        .last_modified()
        .await
        .unwrap();

    let response = check!(helper::get_modified_data(
        Skull::select(store, USER)
            .unwrap()
            .create(<Skull as helper::TesterData>::new(4))
            .await
            .unwrap(),
        skull_last_modified
    ));
    assert_eq!(response, 4);

    assert_eq!(
        Quick::select(store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap(),
        quick_last_modified
    );

    assert_eq!(
        Occurrence::select(store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap(),
        occurrence_last_modified
    );
}

pub async fn delete_cascade(store: &impl Store) {
    helper::populate(store).await;

    for i in 1..=3 {
        Occurrence::select(store, USER)
            .unwrap()
            .delete(i)
            .await
            .unwrap();
    }

    let quick_last_modified = Quick::select(store, USER)
        .unwrap()
        .update(
            1,
            Quick {
                skull: 1,
                amount: 3.,
            },
        )
        .await
        .unwrap()
        .1;

    Skull::select(store, USER).unwrap().delete(1).await.unwrap();

    assert!(
        Quick::select(store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap()
            > quick_last_modified
    );

    assert_eq!(
        Skull::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        helper::from_range::<Skull>(2..=3)
    );

    assert_eq!(
        Quick::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        helper::from_range::<Quick>(2..=2)
    );

    assert!(Occurrence::select(store, USER)
        .unwrap()
        .list(None)
        .await
        .unwrap()
        .0
        .is_empty());
}

pub async fn delete_reject(store: &impl Store) {
    helper::populate(store).await;

    let skull_last_modified = Skull::select(store, USER)
        .unwrap()
        .last_modified()
        .await
        .unwrap();

    let occurrence_last_modified = Occurrence::select(store, USER)
        .unwrap()
        .last_modified()
        .await
        .unwrap();

    let err = Skull::select(store, USER)
        .unwrap()
        .delete(1)
        .await
        .unwrap_err()
        .to_string();
    assert_eq!(err, "Failed constraint");

    assert_eq!(
        Skull::select(store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap(),
        skull_last_modified
    );
    assert_eq!(
        Occurrence::select(store, USER)
            .unwrap()
            .last_modified()
            .await
            .unwrap(),
        occurrence_last_modified
    );

    assert_eq!(
        Skull::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        helper::from_range::<Skull>(1..=3)
    );

    assert_eq!(
        Quick::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        helper::from_range::<Quick>(1..=3)
    );

    assert_eq!(
        Occurrence::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        helper::from_range::<Occurrence>(1..=3)
    );
}

pub async fn multiple_handles(store: impl Store) {
    let store = std::sync::Arc::new(store);
    helper::populate(store.as_ref()).await;

    let cloned_store = store.clone();
    let skull_task = async move {
        let crud = Skull::select(cloned_store.as_ref(), USER).unwrap();
        for i in 1..=3 {
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Skull>(1..=3)
            );
            assert_eq!(
                crud.create(<Skull as helper::TesterData>::new(i + 3))
                    .await
                    .unwrap()
                    .0,
                Id::from(i) + 3
            );
            let mut expected = helper::from_range::<Skull>(1..=3);
            expected.push(<Skull as helper::TesterData>::with_id(i + 3));
            assert_eq!(crud.list(None).await.unwrap().0, expected);
            assert_eq!(
                crud.delete(Id::from(i) + 3).await.unwrap().0,
                <Skull as helper::TesterData>::with_id(i + 3)
            );
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Skull>(1..=3)
            );
        }
    };

    let cloned_store = store.clone();
    let quick_task = async move {
        use crate::store::data::QuickId;
        let crud = Quick::select(cloned_store.as_ref(), USER).unwrap();
        for i in 1..=3 {
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Quick>(1..=3)
            );
            let data = Quick {
                skull: 1,
                amount: 7.,
            };
            assert_eq!(crud.create(data.clone()).await.unwrap().0, i + 3);
            let data: QuickId = WithId::new(i + 3, data);
            let mut expected = helper::from_range::<Quick>(1..=3);
            expected.push(data.clone());
            assert_eq!(crud.list(None).await.unwrap().0, expected);
            assert_eq!(crud.delete(i + 3).await.unwrap().0, data);
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Quick>(1..=3)
            );
        }
    };

    let cloned_store = store.clone();
    let occurrence_task = async move {
        use crate::store::data::OccurrenceId;
        let crud = Occurrence::select(cloned_store.as_ref(), USER).unwrap();
        for i in 1..=3 {
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Occurrence>(1..=3)
            );
            let data = Occurrence {
                skull: 1,
                amount: 2.,
                millis: 3,
            };
            assert_eq!(crud.create(data.clone()).await.unwrap().0, i + 3);
            let data: OccurrenceId = WithId::new(i + 3, data);
            let mut expected = helper::from_range::<Occurrence>(1..=3);
            expected.push(data.clone());
            assert_eq!(crud.list(None).await.unwrap().0, expected);
            assert_eq!(crud.delete(i + 3).await.unwrap().0, data);
            assert_eq!(
                crud.list(None).await.unwrap().0,
                helper::from_range::<Occurrence>(1..=3)
            );
        }
    };

    let poller = helper::Poller::new(
        Box::pin(skull_task),
        Box::pin(quick_task),
        Box::pin(occurrence_task),
    );

    poller.await;
}

mod helper {
    use crate::test_util::Assertion;

    use super::{Id, Occurrence, Quick, Selector, Skull, Store, WithId, USER};

    pub trait TesterData: Selector {
        fn new(i: u8) -> Self;

        fn with_id(i: u8) -> Self::Id {
            Self::Id::new(Id::from(i), Self::new(i))
        }

        fn make_conflicts(&self) -> Vec<Self>;
        fn make_non_conflicts(&self) -> Vec<Self>;
        fn make_unconstraint() -> Option<Self>;
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

        fn make_conflicts(&self) -> Vec<Self> {
            vec![
                self.clone(),
                Self {
                    name: self.name.clone(),
                    ..Self::new(7)
                },
                Self {
                    color: self.color.clone(),
                    ..Self::new(7)
                },
                Self {
                    icon: self.icon.clone(),
                    ..Self::new(7)
                },
            ]
        }

        fn make_non_conflicts(&self) -> Vec<Self> {
            vec![Self {
                unit_price: self.unit_price,
                limit: self.limit,
                ..Self::new(7)
            }]
        }

        fn make_unconstraint() -> Option<Self> {
            None
        }
    }

    impl TesterData for Quick {
        fn new(i: u8) -> Quick {
            Quick {
                skull: u32::from(i),
                amount: f32::from(i),
            }
        }

        fn make_conflicts(&self) -> Vec<Self> {
            vec![self.clone()]
        }

        fn make_non_conflicts(&self) -> Vec<Self> {
            vec![
                Self {
                    skull: self.skull,
                    ..Self::new(7)
                },
                Self {
                    amount: self.amount,
                    ..Self::new(7)
                },
            ]
        }

        fn make_unconstraint() -> Option<Self> {
            Some(Quick {
                skull: 7,
                amount: 7.,
            })
        }
    }

    impl TesterData for Occurrence {
        fn new(i: u8) -> Occurrence {
            Occurrence {
                skull: u32::from(i),
                amount: f32::from(i),
                millis: i64::from(i),
            }
        }

        fn make_conflicts(&self) -> Vec<Self> {
            Vec::new()
        }

        fn make_non_conflicts(&self) -> Vec<Self> {
            Vec::new()
        }

        fn make_unconstraint() -> Option<Self> {
            Some(Occurrence {
                skull: 7,
                amount: 7.,
                millis: 7,
            })
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

    pub struct Poller(
        usize,
        std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
    );

    impl Poller {
        pub fn new(
            f1: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
            f2: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
            f3: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        ) -> Self {
            Self(0, Box::pin(f1), Box::pin(f2), Box::pin(f3))
        }
    }

    impl std::future::Future for Poller {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let unpinned = std::pin::Pin::into_inner(self);

            for _ in 0..3 {
                unpinned.0 = (unpinned.0 + 1) % 3;

                match unpinned.0 {
                    0 => match unpinned.1.as_mut().poll(cx) {
                        std::task::Poll::Ready(_) => continue,
                        std::task::Poll::Pending => return std::task::Poll::Pending,
                    },
                    1 => match unpinned.2.as_mut().poll(cx) {
                        std::task::Poll::Ready(_) => continue,
                        std::task::Poll::Pending => return std::task::Poll::Pending,
                    },
                    2 => match unpinned.3.as_mut().poll(cx) {
                        std::task::Poll::Ready(_) => continue,
                        std::task::Poll::Pending => return std::task::Poll::Pending,
                    },
                    _ => unreachable!(),
                }
            }

            std::task::Poll::Ready(())
        }
    }
}
