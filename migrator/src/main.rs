mod args;

fn main() {
    let args = args::parse();
    match args {
        args::Args::Skull(args::Direction::ToSql(io)) => println!(
            "convert skull to sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
        args::Args::Skull(args::Direction::FromSql(io)) => println!(
            "convert skull from sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
        args::Args::Quick(args::Direction::ToSql(io)) => println!(
            "convert quick to sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
        args::Args::Quick(args::Direction::FromSql(io)) => println!(
            "convert quick from sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
        args::Args::Occurrence(args::Direction::ToSql(io)) => println!(
            "convert occurrence to sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
        args::Args::Occurrence(args::Direction::FromSql(io)) => println!(
            "convert occurrence from sql from {} to {}",
            io.input.display(),
            io.output.display()
        ),
    }
}
