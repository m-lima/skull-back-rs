mod crud;
mod data;
mod error;
mod in_file;
mod in_memory;

pub type Id = u32;
pub use crud::{Crud, Selector, Store};
pub use data::{Data, Occurrence, Quick, Skull, WithId};
pub use error::Error;

pub fn in_memory<S, I>(users: I) -> impl Store
where
    S: ToString,
    I: std::iter::IntoIterator<Item = S>,
{
    in_memory::InMemory::new(users)
}

pub fn in_file<S, I, P>(path: P, users: I) -> anyhow::Result<impl Store>
where
    S: AsRef<str>,
    I: std::iter::IntoIterator<Item = S>,
    P: AsRef<std::path::Path>,
{
    in_file::InFile::new(path, users)
}

#[cfg(all(test, nightly))]
mod bench {
    use super::{Occurrence, Skull, Store};

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

    struct Defer<T: Fn()>(T);

    impl<T: Fn()> Drop for Defer<T> {
        fn drop(&mut self) {
            self.0();
        }
    }

    fn setup_skull(store: &impl Store) {
        store
            .skull(USER)
            .unwrap()
            .write()
            .unwrap()
            .create(Skull {
                name: String::from("skull"),
                color: String::from("color"),
                icon: String::from("icon"),
                unit_price: 1.,
                limit: None,
            })
            .unwrap();
    }

    fn spawn<T: Store>(sender: Sender) -> Vec<std::thread::JoinHandle<()>> {
        let mut threads = Vec::with_capacity(20);

        for _ in 0..10 {
            threads.push(std::thread::spawn(move || {
                let store = sender.get::<T>();
                store
                    .occurrence(USER)
                    .unwrap()
                    .write()
                    .unwrap()
                    .create(OCCURRENCE)
                    .unwrap();
            }));
            threads.push(std::thread::spawn(move || {
                let store = sender.get::<T>();
                store
                    .occurrence(USER)
                    .unwrap()
                    .read()
                    .unwrap()
                    .list(Some(10))
                    .unwrap();
            }));
        }

        threads
    }

    #[bench]
    fn in_memory(bench: &mut test::Bencher) {
        let store = super::in_memory::InMemory::new([USER]);
        setup_skull(&store);
        let sender = Sender::new(&store);

        bench.iter(|| {
            let threads = spawn::<super::in_memory::InMemory>(sender);
            for t in threads {
                t.join().unwrap();
            }
        });
    }

    #[bench]
    fn in_file(bench: &mut test::Bencher) {
        let dir = std::env::temp_dir().join(rand::random::<u64>().to_string());
        std::fs::create_dir(&dir).unwrap();
        let _defer = Defer(|| std::fs::remove_dir_all(&dir).unwrap());
        let store = super::in_file::InFile::new(&dir, [USER]).unwrap();
        setup_skull(&store);
        let sender = Sender::new(&store);

        bench.iter(|| {
            let threads = spawn::<super::in_file::InFile>(sender);
            for t in threads {
                t.join().unwrap();
            }
        });
    }
}
