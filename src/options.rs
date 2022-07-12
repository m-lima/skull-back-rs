pub fn parse() -> Options {
    use clap::Parser;
    Options::parse()
}

#[derive(clap::Parser)]
pub struct Options {
    /// Selects the port to serve on
    #[clap(short, long, default_value = "80")]
    pub port: u16,

    /// Selects the number of threads to use. Zero for automatic
    #[clap(short, long, default_value = "0")]
    pub threads: u8,

    /// Sets the 'allow-origin' header
    #[clap(short, long, parse(try_from_str = to_cors), conflicts_with = "web-path")]
    pub cors: Option<gotham::hyper::header::HeaderValue>,

    /// Sets storage location
    ///
    /// Will store data in memory if no path is provided
    #[clap(short, long, parse(try_from_str = to_dir_path))]
    pub store_path: Option<std::path::PathBuf>,

    /// The directory of the front-end content
    ///
    /// If set, the front-end will be served on the root path "/"
    /// and the api will be nested under "/api"
    #[clap(short, long, parse(try_from_str = to_index_root))]
    pub web_path: Option<std::path::PathBuf>,

    /// Initializes with at least these users present
    #[clap(short, long)]
    pub users: Vec<String>,
}

fn to_cors(
    value: &str,
) -> Result<gotham::hyper::header::HeaderValue, gotham::hyper::header::InvalidHeaderValue> {
    gotham::hyper::header::HeaderValue::from_str(value)
}

fn to_dir_path(value: &str) -> Result<std::path::PathBuf, &'static str> {
    let path = std::path::PathBuf::from(value);
    if !path.is_dir() {
        return Err("path is not a directory");
    }

    Ok(path)
}

fn to_index_root(value: &str) -> Result<std::path::PathBuf, &'static str> {
    let path = std::path::PathBuf::from(value);
    if !path.is_dir() {
        return Err("path is not a directory");
    }

    if !path.join("index.html").exists() {
        return Err("path does not contain an index.html");
    }

    Ok(path)
}
