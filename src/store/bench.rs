use crate::test_util::create_base_test_path;

use super::{Occurrence, Selector, Skull, Store};

extern crate test;

const USER: &str = "bloink";

#[derive(Copy, Clone)]
struct Sender(usize);

impl Sender {
    fn new<T: Store>(store: &T) -> Self {
        Self(store as *const T as usize)
    }

    fn get<T: Store>(&self) -> &T {
        unsafe { &*(self.0 as *const T) }
    }
}

unsafe impl Send for Sender {}
unsafe impl Sync for Sender {}

const OCCURRENCE: Occurrence = Occurrence {
    skull: 1,
    amount: 1.,
    millis: 0,
};

async fn migrate_db(path: &std::path::Path) {
    let path = path.join(USER);
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display()))
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
}

async fn setup_skull<S: Store>(store: S) -> S {
    Skull::select(&store, USER)
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
    store
}

fn spawn<T: Store>(sender: Sender) -> Vec<tokio::task::JoinHandle<()>> {
    let mut tasks = Vec::with_capacity(20);

    for _ in 0..10 {
        tasks.push(tokio::spawn(async move {
            let store = sender.get::<T>();
            Occurrence::select(store, USER)
                .unwrap()
                .create(OCCURRENCE)
                .await
                .unwrap();
        }));
        tasks.push(tokio::spawn(async move {
            let store = sender.get::<T>();
            Occurrence::select(store, USER)
                .unwrap()
                .list(Some(10))
                .await
                .unwrap();
        }));
    }

    tasks
}

fn launch<T: Store>(sender: Sender) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let threads = spawn::<T>(sender);
            for t in threads {
                t.await.unwrap();
            }
        });
}

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[bench]
fn in_memory(bench: &mut test::Bencher) {
    let store = build_runtime().block_on(async {
        let store = super::in_memory::InMemory::new([USER]);

        setup_skull(store).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        launch::<super::in_memory::InMemory>(sender);
    });
}

#[bench]
fn in_file(bench: &mut test::Bencher) {
    let path = create_base_test_path();
    let store = build_runtime().block_on(async {
        let store = super::in_file::InFile::new(
            Some((String::from(USER), path.join(USER)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        setup_skull(store).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        launch::<super::in_file::InFile>(sender);
    });
}

#[bench]
fn in_db(bench: &mut test::Bencher) {
    let path = create_base_test_path();
    let store = build_runtime().block_on(async {
        let store = super::in_db::InDb::new(
            Some((String::from(USER), path.join(USER)))
                .into_iter()
                .collect(),
        )
        .unwrap();

        migrate_db(&path).await;
        setup_skull(store).await
    });

    let sender = Sender::new(&store);
    bench.iter(|| {
        launch::<super::in_file::InFile>(sender);
    });
}
