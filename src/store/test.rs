#[macro_export]
macro_rules! impl_crud_tests {
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
                async fn multiple_spawned_handles() {
                    $crate::store::test::multiple_spawned_handles($instance).await;
                }

                #[tokio::test(flavor = "multi_thread")]
                async fn multiple_polled_handles() {
                    $crate::store::test::multiple_polled_handles($instance).await;
                }
            }

            $crate::impl_crud_tests!(skull, $crate::store::Skull, $uut, $instance);
            $crate::impl_crud_tests!(quick, $crate::store::Quick, $uut, $instance);
            $crate::impl_crud_tests!(occurrence, $crate::store::Occurrence, $uut, $instance);
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

use super::{Crud, Error, Id, Occurrence, Quick, Selector, Skull, Store, WithId};
use helper::{Poller, TesterData};

pub const USER: &str = "bloink";

pub struct Tester<D: Selector>(std::marker::PhantomData<D>);

impl<D: TesterData> Tester<D> {
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
        check!(D::compare_with_range(response, 1..=3));
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
        check!(D::compare_limited(response, 1..=3, 1));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(2)).await.unwrap(),
            last_modified
        ));
        check!(D::compare_limited(response, 1..=3, 2));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(3)).await.unwrap(),
            last_modified
        ));
        check!(D::compare_with_range(response, 1..=3));

        let response = check!(helper::get_unchanged_data(
            store.list(Some(4)).await.unwrap(),
            last_modified
        ));
        check!(D::compare_with_range(response, 1..=3));
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
        let data = D::new(7).with_skull(1);
        let response = check!(helper::get_modified_data(
            store.create(data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 4);

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected.push(D::Id::new(4, data));
        check!(D::compare(response, expected));
    }

    pub async fn create_conflict(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let initial = D::new(4).with_skull(1);
        let last_modified = store.create(initial.clone()).await.unwrap().1;
        let mut expected = helper::from_range::<D>(1..=3);
        expected.push(D::Id::new(4, initial.clone()));

        for conflicting in initial.make_conflicts() {
            let err = store.create(conflicting).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Conflicting entry");
        }

        for (id, non_conflicting) in initial.make_non_conflicts().into_iter().enumerate() {
            let response = check!(helper::get_modified_data(
                store.create(non_conflicting.clone()).await.unwrap(),
                last_modified
            ));
            let expected_id = Id::try_from(id + 5).unwrap();
            assert_eq!(response, expected_id);
            expected.push(D::Id::new(expected_id, non_conflicting));
        }

        let response = store.list(None).await.unwrap().0;
        check!(D::compare(response, expected));
    }

    pub async fn create_constraint(store: &impl Store) {
        if let Some(unconstrained) = D::make_unconstrained() {
            helper::populate(store).await;
            let store = D::select(store, USER).unwrap();

            let last_modified = store.last_modified().await.unwrap();

            let err = store.create(unconstrained).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");

            let response = store.list(None).await.unwrap().0;
            check!(D::compare_with_range(response, 1..=3));
        }
    }

    pub async fn create_monotonic(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        for i in 4..=21 {
            store.create(D::new(i).with_skull(1)).await.unwrap();
        }

        for i in 4..=21 {
            if i % 3 == 0 || i % 4 == 0 {
                store.delete(i).await.unwrap();
            }
        }

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_modified_data(
            store.create(D::new(4).with_skull(1)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, 20);
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
            assert_eq!(response, D::ided(i));
        }
    }

    pub async fn read_sparse(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        for i in 4..=20 {
            store.create(D::new(i).with_skull(1)).await.unwrap();
        }

        for i in 4..=20 {
            if i % 3 == 0 || i % 4 == 0 {
                store.delete(i).await.unwrap();
            }
        }

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_unchanged_data(
            store.read(11).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::new(11).with_skull(1).with_id(11));
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

        let data = D::new(7).with_skull(1);
        let response = check!(helper::get_modified_data(
            store.update(2, data.clone()).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::ided(2));

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected[1] = D::Id::new(2, data);
        check!(D::compare(response, expected));
    }

    pub async fn update_no_changes(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let last_modified = store.last_modified().await.unwrap();

        let response = check!(helper::get_unchanged_data(
            store.update(2, D::new(2)).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::ided(2));

        let response = store.list(None).await.unwrap().0;
        check!(D::compare_with_range(response, 1..=3));
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
        check!(D::compare_with_range(response, 1..=3));
    }

    pub async fn update_conflict(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        let initial = D::new(4).with_skull(1);

        store.create(initial.clone()).await.unwrap();
        let mut last_modified = store.create(D::new(5).with_skull(1)).await.unwrap().1;

        let mut expected = helper::from_range::<D>(1..=3);
        expected.push(initial.clone().with_id(4));
        expected.push(D::new(5).with_skull(1).with_id(5));

        for (i, _) in initial.make_non_conflicts().iter().enumerate() {
            last_modified = store
                .create(D::new(u8::try_from(i + 6).unwrap()).with_skull(3))
                .await
                .unwrap()
                .1;
        }

        for conflicting in initial.make_conflicts() {
            let err = store.update(2, conflicting).await.unwrap_err().to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Conflicting entry");
        }

        for (id, non_conflicting) in initial.make_non_conflicts().into_iter().enumerate() {
            let id = Id::try_from(id + 6).unwrap();
            let id_u8 = u8::try_from(id).unwrap();
            let response = check!(helper::get_modified_data(
                store.update(id, non_conflicting.clone()).await.unwrap(),
                last_modified
            ));
            assert_eq!(response, D::new(id_u8).with_skull(3).with_id(id_u8));
            expected.push(D::Id::new(id, non_conflicting));
        }

        let response = store.list(None).await.unwrap().0;
        check!(D::compare(response, expected));
    }

    pub async fn update_constraint(store: &impl Store) {
        if let Some(unconstrained) = D::make_unconstrained() {
            helper::populate(store).await;
            let store = D::select(store, USER).unwrap();

            let last_modified = store.last_modified().await.unwrap();

            let err = store
                .update(2, unconstrained)
                .await
                .unwrap_err()
                .to_string();
            assert_eq!(store.last_modified().await.unwrap(), last_modified);
            assert_eq!(err, "Failed constraint");

            let response = store.list(None).await.unwrap().0;
            check!(D::compare_with_range(response, 1..=3));
        }
    }

    pub async fn delete(store: &impl Store) {
        helper::populate(store).await;
        let store = D::select(store, USER).unwrap();

        store.create(D::new(4).with_skull(1)).await.unwrap();
        let last_modified = store.create(D::new(5).with_skull(1)).await.unwrap().1;

        let response = check!(helper::get_modified_data(
            store.delete(4).await.unwrap(),
            last_modified
        ));
        assert_eq!(response, D::new(4).with_skull(1).with_id(4));

        let response = store.list(None).await.unwrap().0;
        let mut expected = helper::from_range::<D>(1..=3);
        expected.push(D::new(5).with_skull(1).with_id(5));
        check!(D::compare(response, expected));
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
        check!(D::compare_with_range(response, 1..=3));
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
            .create(Skull::new(4))
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
            3,
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

    check!(Skull::compare_with_range(
        Skull::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        2..=3
    ));

    check!(Quick::compare_with_range(
        Quick::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        2..=2
    ));

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

    check!(Skull::compare_with_range(
        Skull::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        1..=3
    ));

    check!(Quick::compare_with_range(
        Quick::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        1..=3
    ));

    check!(Occurrence::compare_with_range(
        Occurrence::select(store, USER)
            .unwrap()
            .list(None)
            .await
            .unwrap()
            .0,
        1..=3
    ));
}

pub async fn multiple_spawned_handles(store: impl Store) {
    let (skull_task, quick_task, occurrence_task) = helper::make_futures(store).await;

    for t in vec![
        tokio::spawn(skull_task),
        tokio::spawn(quick_task),
        tokio::spawn(occurrence_task),
    ] {
        t.await.unwrap();
    }
}

pub async fn multiple_polled_handles(store: impl Store) {
    let (skull_task, quick_task, occurrence_task) = helper::make_futures(store).await;

    let poller = Poller::new(
        Box::pin(skull_task),
        Box::pin(quick_task),
        Box::pin(occurrence_task),
    );

    poller.await;
}

mod helper {
    use crate::test_util::Assertion;

    use super::{check, Crud, Id, Occurrence, Quick, Selector, Skull, Store, WithId, USER};

    pub trait TesterData: Selector {
        fn new(i: u8) -> Self;

        fn ided(i: u8) -> Self::Id {
            Self::new(i).with_id(i)
        }

        fn with_id(self, i: u8) -> Self::Id {
            Self::Id::new(Id::from(i), self)
        }

        fn with_skull(self, id: Id) -> Self;

        fn make_conflicts(&self) -> Vec<Self>;
        fn make_non_conflicts(&self) -> Vec<Self>;
        fn make_unconstrained() -> Option<Self>;

        fn compare(got: Vec<Self::Id>, wanted: Vec<Self::Id>) -> Assertion<()>;
        fn compare_with_range(
            got: Vec<Self::Id>,
            wanted: std::ops::RangeInclusive<u8>,
        ) -> Assertion<()> {
            Self::compare(got, from_range::<Self>(wanted))
        }
        fn compare_limited(
            got: Vec<Self::Id>,
            wanted: std::ops::RangeInclusive<u8>,
            count: usize,
        ) -> Assertion<()>;
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

        fn with_skull(self, _id: Id) -> Self {
            self
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

        fn make_unconstrained() -> Option<Self> {
            None
        }

        fn compare(mut got: Vec<Self::Id>, mut wanted: Vec<Self::Id>) -> Assertion<()> {
            got.sort_unstable_by_key(WithId::id);
            wanted.sort_unstable_by_key(WithId::id);

            if got == wanted {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output arrays did not match", got, wanted)
            }
        }

        fn compare_limited(
            got: Vec<Self::Id>,
            _wanted: std::ops::RangeInclusive<u8>,
            count: usize,
        ) -> Assertion<()> {
            if got.len() == count {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output array size did not match", got.len(), count)
            }
        }
    }

    impl TesterData for Quick {
        fn new(i: u8) -> Quick {
            Quick {
                skull: u32::from(i),
                amount: f32::from(i),
            }
        }

        fn with_skull(self, id: Id) -> Self {
            Self { skull: id, ..self }
        }

        fn make_conflicts(&self) -> Vec<Self> {
            vec![self.clone()]
        }

        fn make_non_conflicts(&self) -> Vec<Self> {
            vec![
                Self {
                    skull: self.skull,
                    amount: 7.,
                },
                Self {
                    skull: ((self.skull + 1) % 3) + 1,
                    amount: self.amount,
                },
            ]
        }

        fn make_unconstrained() -> Option<Self> {
            Some(Quick {
                skull: 7,
                amount: 7.,
            })
        }

        fn compare(mut got: Vec<Self::Id>, mut wanted: Vec<Self::Id>) -> Assertion<()> {
            got.sort_unstable_by_key(WithId::id);
            wanted.sort_unstable_by_key(WithId::id);

            if got == wanted {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output arrays did not match", got, wanted)
            }
        }

        fn compare_limited(
            got: Vec<Self::Id>,
            _wanted: std::ops::RangeInclusive<u8>,
            count: usize,
        ) -> Assertion<()> {
            if got.len() == count {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output array size did not match", got.len(), count)
            }
        }
    }

    impl TesterData for Occurrence {
        fn new(i: u8) -> Occurrence {
            use rand::{RngCore, SeedableRng};
            Occurrence {
                skull: u32::from(i),
                amount: f32::from(i),
                millis: i64::from(rand::rngs::StdRng::from_seed([i; 32]).next_u32()),
            }
        }

        fn with_skull(self, id: Id) -> Self {
            Self { skull: id, ..self }
        }

        fn make_conflicts(&self) -> Vec<Self> {
            Vec::new()
        }

        fn make_non_conflicts(&self) -> Vec<Self> {
            Vec::new()
        }

        fn make_unconstrained() -> Option<Self> {
            Some(Occurrence {
                skull: 7,
                amount: 7.,
                millis: 7,
            })
        }

        fn compare(got: Vec<Self::Id>, mut wanted: Vec<Self::Id>) -> Assertion<()> {
            wanted.sort_unstable_by(|a, b| match b.millis.cmp(&a.millis) {
                std::cmp::Ordering::Equal => b.id.cmp(&a.id),
                c => c,
            });

            if got == wanted {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output arrays did not match", got, wanted)
            }
        }

        fn compare_limited(
            got: Vec<Self::Id>,
            wanted: std::ops::RangeInclusive<u8>,
            count: usize,
        ) -> Assertion<()> {
            let mut wanted = from_range::<Self>(wanted);

            wanted.sort_unstable_by(|a, b| match b.millis.cmp(&a.millis) {
                std::cmp::Ordering::Equal => b.id.cmp(&a.id),
                c => c,
            });

            let wanted = wanted.into_iter().take(count).collect::<Vec<_>>();

            if got == wanted {
                Assertion::Ok(())
            } else {
                Assertion::err_ne("Output arrays did not match", got, wanted)
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
        range.map(D::ided).collect()
    }

    pub async fn make_futures<S: Store>(
        store: S,
    ) -> (
        impl std::future::Future<Output = ()>,
        impl std::future::Future<Output = ()>,
        impl std::future::Future<Output = ()>,
    ) {
        let store = std::sync::Arc::new(store);
        populate(store.as_ref()).await;

        let cloned_store = store.clone();
        let skull_task = async move {
            let crud = Skull::select(cloned_store.as_ref(), USER).unwrap();
            for _ in 1..=3 {
                check!(Skull::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
                assert_eq!(crud.create(Skull::new(4)).await.unwrap().0, 4);
                check!(Skull::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=4
                ));
                assert_eq!(crud.delete(4).await.unwrap().0, Skull::ided(4));
                check!(Skull::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
            }
        };

        let cloned_store = store.clone();
        let quick_task = async move {
            use crate::store::data::QuickId;
            let crud = Quick::select(cloned_store.as_ref(), USER).unwrap();
            for _ in 1..=3 {
                check!(Quick::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
                let data = Quick {
                    skull: 1,
                    amount: 7.,
                };
                assert_eq!(crud.create(data.clone()).await.unwrap().0, 4);
                let data: QuickId = WithId::new(4, data);
                let mut expected = from_range::<Quick>(1..=3);
                expected.push(data.clone());
                check!(Quick::compare(crud.list(None).await.unwrap().0, expected));
                assert_eq!(crud.delete(4).await.unwrap().0, data);
                check!(Quick::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
            }
        };

        let cloned_store = store.clone();
        let occurrence_task = async move {
            use crate::store::data::OccurrenceId;
            let crud = Occurrence::select(cloned_store.as_ref(), USER).unwrap();
            for _ in 1..=3 {
                check!(Occurrence::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
                let data = Occurrence {
                    skull: 1,
                    amount: 2.,
                    millis: 3,
                };
                assert_eq!(crud.create(data.clone()).await.unwrap().0, 4);
                let data: OccurrenceId = WithId::new(4, data);
                let mut expected = from_range::<Occurrence>(1..=3);
                expected.push(data.clone());
                check!(Occurrence::compare(
                    crud.list(None).await.unwrap().0,
                    expected
                ));
                assert_eq!(crud.delete(4).await.unwrap().0, data);
                check!(Occurrence::compare_with_range(
                    crud.list(None).await.unwrap().0,
                    1..=3
                ));
            }
        };

        (skull_task, quick_task, occurrence_task)
    }

    pub struct Poller(Vec<std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>>);

    impl Poller {
        pub fn new(
            f1: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
            f2: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
            f3: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>,
        ) -> Self {
            Self(vec![Box::pin(f1), Box::pin(f2), Box::pin(f3)])
        }
    }

    impl std::future::Future for Poller {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let unpinned = std::pin::Pin::into_inner(self);

            loop {
                if unpinned.0.is_empty() {
                    return std::task::Poll::Ready(());
                }

                let index = rand::random::<usize>() % unpinned.0.len();
                match unpinned.0[index].as_mut().poll(cx) {
                    std::task::Poll::Ready(_) => {
                        unpinned.0.remove(index);
                    }
                    std::task::Poll::Pending => return std::task::Poll::Pending,
                }
            }
        }
    }
}
