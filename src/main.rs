#![deny(warnings, clippy::pedantic, clippy::all)]
#![warn(rust_2018_idioms)]
#![cfg_attr(all(test, nightly), feature(test))]

mod options;
mod server;
mod store;

fn init_logger() {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_str("%Y-%m-%dT%H:%M:%SZ")
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
        log::error!("Could not initialize router: {}", e);
        std::process::exit(-1);
    });

    if threads > 0 {
        let threads = usize::from(threads);
        log::info!("Core threads set to {}", threads);
        gotham::start_with_num_threads(format!("0.0.0.0:{}", port), route, threads);
    } else {
        log::info!("Core threads set to automatic");
        gotham::start(format!("0.0.0.0:{}", port), route);
    }
}
