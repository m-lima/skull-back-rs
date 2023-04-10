#![allow(clippy::missing_panics_doc)]

#[macro_export]
macro_rules! check_sync {
    ($assertion: expr) => {
        $assertion.assert(concat!(file!(), ":", line!(), ":", column!()))
    };
}

#[macro_export]
macro_rules! check_async {
    ($assertion: expr, $runtime: expr) => {
        $runtime
            .block_on($assertion)
            .assert(concat!(file!(), ":", line!(), ":", column!()))
    };
    ($assertion: expr) => {
        $assertion
            .await
            .assert(concat!(file!(), ":", line!(), ":", column!()))
    };
}

pub const USER: &str = "bloink";

pub struct TestPath(std::path::PathBuf);

impl TestPath {
    #[must_use]
    pub fn new() -> Self {
        let name = format!(
            "{:016x}{:016x}",
            rand::random::<u64>(),
            rand::random::<u64>()
        );
        let path = std::env::temp_dir().join("skull-test");
        if path.exists() {
            assert!(path.is_dir(), "Cannot use {} as test path", path.display());
        } else {
            std::fs::create_dir(&path).unwrap();
        }
        let path = path.join(name);
        assert!(
            !path.exists(),
            "Cannot use {} as test path as it already exists",
            path.display()
        );
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Default for TestPath {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for TestPath {
    type Target = std::path::PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TestPath {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

pub enum Assertion<T> {
    Ok(T),
    Err(&'static str, String, Option<String>),
}

impl<T> Assertion<T> {
    pub fn err_ne(
        message: &'static str,
        got: impl std::fmt::Debug,
        wanted: impl std::fmt::Debug,
    ) -> Self {
        Self::Err(message, format!("{got:?}"), Some(format!("{wanted:?}")))
    }

    pub fn err_eq(message: &'static str, got: impl std::fmt::Debug) -> Self {
        Self::Err(message, format!("{got:?}"), None)
    }

    pub fn assert(self, location: &'static str) -> T {
        match self {
            Self::Ok(r) => r,
            Self::Err(message, got, wanted) => {
                eprintln!("{message}");
                eprintln!("Got:    {got}");
                if let Some(wanted) = wanted {
                    eprintln!("Wanted: {wanted}");
                }
                panic!("{location}");
            }
        }
    }
}
