pub fn parse() -> Args {
    use clap::Parser;
    Args::parse()
}

#[derive(clap::Parser)]
#[clap(name = "Migrator", about = "Migrator from TSV to SQL")]
pub struct Args {
    #[arg(short, long, value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_dir_path))]
    pub input: std::path::PathBuf,
    #[arg(short, long, value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_file_path))]
    pub output: std::path::PathBuf,
}

fn to_dir_path(path: std::path::PathBuf) -> Result<std::path::PathBuf, &'static str> {
    if !path.is_dir() {
        return Err("path is not a directory");
    }

    Ok(path)
}

fn to_file_path(path: std::path::PathBuf) -> Result<std::path::PathBuf, String> {
    if path.exists() {
        return Err(String::from("file already exists"));
    }

    std::fs::write(&path, []).map_err(|e| e.to_string())?;

    Ok(path)
}
