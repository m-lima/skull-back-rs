mod args;

fn init_logger(level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(time::macros::format_description!(
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

    simplelog::TermLogger::init(level, config, simplelog::TerminalMode::Mixed, color_choice)
}

fn main() -> std::process::ExitCode {
    let args = args::parse();

    if let Err(err) = init_logger(args.verbosity()) {
        eprintln!("{err}");
        return std::process::ExitCode::FAILURE;
    }

    std::process::ExitCode::SUCCESS
}
