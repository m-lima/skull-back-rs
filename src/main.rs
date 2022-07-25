#![deny(warnings, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]
#![cfg_attr(all(test, nightly), feature(test))]

mod options;
mod server;
mod store;

fn init_logger() {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(simplelog::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
        ))
        .build();

    let color_choice = std::env::var("CLICOLOR_FORCE")
        .ok()
        .filter(|force| force != "0")
        .map(|_| simplelog::ColorChoice::Always)
        .or_else(|| {
            std::env::var("CLICOLOR")
                .ok()
                .filter(|clicolor| clicolor == "0")
                .map(|_| simplelog::ColorChoice::Never)
        })
        .unwrap_or(simplelog::ColorChoice::Auto);

    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        simplelog::LevelFilter::Debug,
        #[cfg(not(debug_assertions))]
        simplelog::LevelFilter::Info,
        config,
        simplelog::TerminalMode::Mixed,
        color_choice,
    )
    .expect("Could not initialize logger");
}

fn main() {
    let options = options::parse();
    init_logger();

    let port = options.port;
    let threads = options.threads;
    let route = server::route(options).unwrap_or_else(|e| {
        log::error!("Could not initialize router: {e}");
        std::process::exit(-1);
    });

    if let Err(e) = if threads > 0 {
        let threads = usize::from(threads);
        log::info!("Core threads set to {threads}");
        gotham::start_with_num_threads(format!("0.0.0.0:{port}"), route, threads)
    } else {
        log::info!("Core threads set to automatic");
        gotham::start(format!("0.0.0.0:{port}"), route)
    } {
        log::error!("Could not start server: {e}");
        std::process::exit(-2);
    }
}

#[cfg(test)]
mod test_util {
    #[macro_export]
    macro_rules! check {
        ($assertion:expr) => {
            $assertion.assert(concat!(file!(), ":", line!(), ":", column!()))
        };
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

    pub struct TestPath(std::path::PathBuf);

    impl TestPath {
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
}
