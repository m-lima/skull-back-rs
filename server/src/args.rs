#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Path does not exist")]
    PathDoesNotExist,
    #[error("Path is not a directory")]
    PathNotDir,
    #[error("Could not open file")]
    CouldNotOpenFile,
    #[error("Expected `auto` or a value in the [1..=255] range")]
    #[cfg(feature = "threads")]
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
    #[arg(short, long)]
    create: bool,

    #[command(flatten)]
    users: Users,

    /// Path to databases directory
    #[arg(value_parser = parse_db)]
    db: std::path::PathBuf,
}

impl Args {
    fn verbosity(&self) -> Verbosity {
        let (level, include_spans) = match self.verbosity {
            0 => (tracing::Level::ERROR, false),
            1 => (tracing::Level::WARN, false),
            2 => (tracing::Level::INFO, false),
            3 => (tracing::Level::INFO, true),
            4 => (tracing::Level::DEBUG, true),
            _ => (tracing::Level::TRACE, true),
        };

        Verbosity {
            level,
            include_spans,
        }
    }

    pub fn decompose(
        self,
    ) -> (
        Verbosity,
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

#[derive(Debug, Copy, Clone)]
pub struct Verbosity {
    pub level: tracing::Level,
    pub include_spans: bool,
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
