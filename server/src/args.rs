#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Path does not exist")]
    PathDoesNotExist,
    #[error("Path is not a directory")]
    PathNotDir,
    #[error("Could not open file")]
    CouldNotOpenFile,
    #[error("Expected `auto` or a value in the [1..=255] range")]
    Threads,
}

pub fn parse() -> Args {
    <Args as clap::Parser>::parse()
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Verbosity level
    #[arg(short, action = clap::ArgAction::Count)]
    verbosity: u8,

    /// Port to serve on
    #[arg(short, long, default_value = "80", value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// Number of threads
    #[arg(short, long, default_value = "auto", value_parser = boile_rs::rt::threads::parse)]
    #[cfg(feature = "threads")]
    threads: boile_rs::rt::Threads,

    /// Create the databases if they don't exist
    #[arg(short = 'c', long)]
    create: bool,

    #[command(flatten)]
    users: Users,

    /// Path to databases directory
    #[arg(value_parser = parse_db)]
    db: std::path::PathBuf,
}

impl Args {
    fn verbosity(&self) -> tracing::Level {
        match self.verbosity {
            0 => tracing::Level::ERROR,
            1 => tracing::Level::WARN,
            2 => tracing::Level::INFO,
            3 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    }

    pub fn decompose(
        self,
    ) -> (
        tracing::Level,
        u16,
        boile_rs::rt::Threads,
        bool,
        std::path::PathBuf,
        std::collections::HashSet<String>,
    ) {
        (
            self.verbosity(),
            self.port,
            #[cfg(feature = "threads")]
            self.threads,
            #[cfg(not(feature = "threads"))]
            boile_rs::rt::Threads::Single,
            self.create,
            self.db,
            self.users.users(),
        )
    }
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true)]
struct Users {
    /// Initialize with the given users present
    #[arg(short = 'U', long, value_delimiter = ',')]
    add_user: Vec<String>,

    /// Path to the file with the list of users with which to initialize
    ///
    /// File should list one user per line
    /// Lines are trimmed and ignored if empty or starting with `#`
    #[arg(short, long, value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_user_list))]
    users: Option<std::collections::HashSet<String>>,
}

impl Users {
    fn users(self) -> std::collections::HashSet<String> {
        let mut users = self.users.unwrap_or_default();
        users.extend(self.add_user);
        users
    }
}

fn parse_db(input: &str) -> Result<std::path::PathBuf, Error> {
    let input = input.strip_prefix("sqlite://").unwrap_or(input);
    let path = std::path::PathBuf::from(input);

    if !path.exists() {
        Err(Error::PathDoesNotExist)
    } else if !path.is_dir() {
        Err(Error::PathNotDir)
    } else {
        Ok(path)
    }
}

fn to_user_list(path: std::path::PathBuf) -> Result<std::collections::HashSet<String>, Error> {
    let content = std::fs::read_to_string(path).map_err(|_| Error::CouldNotOpenFile)?;
    let mut users = std::collections::HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        users.insert(String::from(line));
    }

    Ok(users)
}

// #[derive(Copy, Clone, Eq, PartialEq)]
// pub enum Threads {
//     Single,
//     Auto,
//     Multi(u16),
// }
//
// impl Threads {
//     fn parse(input: &str) -> Result<Self, Error> {
//         if input == "auto" {
//             Ok(Self::Auto)
//         } else {
//             input.parse().map_err(|_| Error::Threads).and_then(|count| {
//                 if count == 0 {
//                     Err(Error::Threads)
//                 } else if count == 1 {
//                     Ok(Self::Single)
//                 } else {
//                     Ok(Self::Multi(count))
//                 }
//             })
//         }
//     }
// }
//
// impl std::fmt::Display for Threads {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Single => f.write_str("Single"),
//             Self::Auto => f.write_str("Auto"),
//             Self::Multi(count) => write!(f, "Multi({count})"),
//         }
//     }
// }
//
// impl std::fmt::Debug for Threads {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         std::fmt::Display::fmt(self, f)
//     }
// }
