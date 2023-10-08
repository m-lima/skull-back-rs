pub fn parse() -> Options {
    use clap::Parser;
    Options::parse()
}

#[derive(clap::Parser)]
#[clap(name = "Skull", about = "A server for skull book keeping", group = clap::ArgGroup::new("store"))]
pub struct Options {
    /// Selects the port to serve on
    #[arg(short, long, default_value = "80")]
    pub port: u16,

    /// Selects the number of threads to use. Zero for automatic
    #[arg(short, long, default_value = "0")]
    pub threads: u8,

    /// Sets the 'allow-origin' header
    #[arg(short, long, conflicts_with = "web_path", value_parser = to_cors)]
    pub cors: Option<gotham::hyper::header::HeaderValue>,

    /// Sets file storage location
    ///
    /// Creates a file per user in the given directory. If no path is provided,
    /// store data in memory
    #[arg(short, long, group = "store", value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_dir_path))]
    pub store_path: Option<std::path::PathBuf>,

    /// Sets database storage location
    ///
    /// Creates a database per user in the given directory. If no path is provided,
    /// store data in memory
    #[arg(short, long, group = "store", value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_dir_path))]
    pub db_path: Option<std::path::PathBuf>,

    /// The directory of the front-end content
    ///
    /// If set, the front-end will be served on the root path "/"
    /// and the api will be nested under "/api"
    #[arg(short, long, value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_index_root))]
    pub web_path: Option<std::path::PathBuf>,

    /// Initializes with at least these users present
    #[arg(short, long)]
    pub users: Vec<String>,
}

fn to_cors(
    value: &str,
) -> Result<gotham::hyper::header::HeaderValue, gotham::hyper::header::InvalidHeaderValue> {
    gotham::hyper::header::HeaderValue::from_str(value)
}

fn to_dir_path(path: std::path::PathBuf) -> Result<std::path::PathBuf, &'static str> {
    if !path.is_dir() {
        return Err("path is not a directory");
    }

    Ok(path)
}

fn to_index_root(path: std::path::PathBuf) -> Result<std::path::PathBuf, &'static str> {
    if !path.is_dir() {
        return Err("path is not a directory");
    }

    if !path.join("index.html").exists() {
        return Err("path does not contain an index.html");
    }

    Ok(path)
}
