pub fn parse() -> Args {
    use clap::Parser;
    Args::parse()
}

#[derive(clap::Parser)]
#[clap(name = "Migrator", about = "Migrator between TSV and SQL")]
pub enum Args {
    #[command(subcommand)]
    Skull(Direction),
    #[command(subcommand)]
    Quick(Direction),
    #[command(subcommand)]
    Occurrence(Direction),
}

#[derive(clap::Subcommand)]
pub enum Direction {
    FromSql(IO),
    ToSql(IO),
}

#[derive(Debug, clap::Args)]
pub struct IO {
    #[arg(value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_file_path))]
    pub input: std::path::PathBuf,

    #[arg(value_parser = clap::builder::TypedValueParser::try_map(clap::builder::PathBufValueParser::new(), to_file_path))]
    pub output: std::path::PathBuf,
}

fn to_file_path(path: std::path::PathBuf) -> Result<std::path::PathBuf, &'static str> {
    if !path.is_file() {
        return Err("path is not a file");
    }

    Ok(path)
}
