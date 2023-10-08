use test_utils::TestPath;

use super::{Crud, Model, Occurrence, Skull, Store};

extern crate test;

const USER1: &str = "bloinker";
const USER2: &str = "bloinkee";
const USER3: &str = "bloinked";

struct Sender<T>(usize, std::marker::PhantomData<T>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0, std::marker::PhantomData)
    }
}

impl<T> Copy for Sender<T> {}

impl<T> Sender<T> {
    fn new(store: &T) -> Self {
        Self(store as *const T as usize, std::marker::PhantomData)
    }

    fn get(&self) -> &T {
        unsafe { &*(self.0 as *const T) }
    }
}

unsafe impl<T> Send for Sender<T> {}
unsafe impl<T> Sync for Sender<T> {}

const OCCURRENCE: Occurrence = Occurrence {
    skull: 1,
    amount: 1.,
    millis: 0,
};

async fn migrate_db<const N: usize>(path: &std::path::Path, users: [&str; N]) {
    async fn inner(path: std::path::PathBuf) {
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display()))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
    }
    for user in users {
        inner(path.join(user)).await;
    }
}

async fn setup_skull<S: Store, const N: usize>(store: S, users: [&str; N]) -> S {
    async fn inner<S: Store>(store: &S, user: &str) {
        Skull::select(store, user)
            .unwrap()
            .create(Skull {
                name: String::from("skull"),
                color: String::from("color"),
                icon: String::from("icon"),
                unit_price: 1.,
                limit: None,
            })
            .await
            .unwrap();
    }
    for user in users {
        inner(&store, user).await;
    }
    store
}

async fn spawn<T: Store, const N: usize>(sender: Sender<T>, users: [&'static str; N]) {
    let mut tasks = Vec::with_capacity(4 * N);

    for i in 0..4_u8 {
        for user in users {
            if i >= 1 && i < 3 {
                tasks.push(tokio::spawn(async move {
                    let store = sender.get();
                    Occurrence::select(store, user)
                        .unwrap()
                        .create(OCCURRENCE)
                        .await
                        .unwrap();
                }));
            }
            tasks.push(tokio::spawn(async move {
                let store = sender.get();
                Occurrence::select(store, user)
                    .unwrap()
                    .list(Some(10))
                    .await
                    .unwrap();
            }));
        }
    }

    for t in tasks {
        t.await.unwrap();
    }
}

async fn spawn_seq<T: Store>(sender: Sender<T>, i: usize) {
    let user = match i % 3 {
        0 => USER1,
        1 => USER2,
        2 => USER3,
        _ => unreachable!(),
    };

    Occurrence::select(sender.get(), user)
        .unwrap()
        .create(OCCURRENCE)
        .await
        .unwrap();
    Occurrence::select(sender.get(), user)
        .unwrap()
        .list(Some(10))
        .await
        .unwrap();
}

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[bench]
fn in_memory_single(bench: &mut test::Bencher) {
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_memory::InMemory::new([USER1]);

        setup_skull(store, [USER1]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1]));
    });
}

#[bench]
fn in_file_single(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_file::InFile::new(
            [USER1]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        setup_skull(store, [USER1]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1]));
    });
}

#[bench]
fn in_db_single(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_db::InDb::new(
            [USER1]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        migrate_db(&path, [USER1]).await;
        setup_skull(store, [USER1]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1]));
    });
}

#[bench]
fn in_memory_multi(bench: &mut test::Bencher) {
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_memory::InMemory::new([USER1, USER2, USER3]);

        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1, USER2, USER3]));
    });
}

#[bench]
fn in_file_multi(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_file::InFile::new(
            [USER1, USER2, USER3]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1, USER2, USER3]));
    });
}

#[bench]
fn in_db_multi(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_db::InDb::new(
            [USER1, USER2, USER3]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        migrate_db(&path, [USER1, USER2, USER3]).await;
        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        runtime.block_on(spawn(sender, [USER1, USER2, USER3]));
    });
}

#[bench]
fn in_memory_seq(bench: &mut test::Bencher) {
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_memory::InMemory::new([USER1, USER2, USER3]);

        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    let mut i = 0;
    bench.iter(|| {
        runtime.block_on(spawn_seq(sender, i));
        i += 1;
    });
}

#[bench]
fn in_file_seq(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_file::InFile::new(
            [USER1, USER2, USER3]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    let mut i = 0;
    bench.iter(|| {
        runtime.block_on(spawn_seq(sender, i));
        i += 1;
    });
}

#[bench]
fn in_db_seq(bench: &mut test::Bencher) {
    let path = TestPath::new();
    let runtime = build_runtime();
    let store = runtime.block_on(async {
        let store = super::in_db::InDb::new(
            [USER1, USER2, USER3]
                .map(|u| (String::from(u), path.join(u)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        migrate_db(&path, [USER1, USER2, USER3]).await;
        setup_skull(store, [USER1, USER2, USER3]).await
    });

    let sender = Sender::new(&store);
    let mut i = 0;
    bench.iter(|| {
        runtime.block_on(spawn_seq(sender, i));
        i += 1;
    });
}
