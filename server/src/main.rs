mod args;
mod layer;
mod router;
mod service;
mod ws;

fn setup_tracing(
    verbosity: args::Verbosity,
) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry().with(boile_rs::log::tracing::layer());

    if verbosity.include_spans {
        let subscriber = subscriber.with(::tracing::level_filters::LevelFilter::from_level(
            verbosity.level,
        ));
        tracing::subscriber::set_global_default(subscriber).map_err(Into::into)
    } else {
        let subscriber = subscriber.with(
            tracing_subscriber::filter::Targets::new()
                .with_default(verbosity.level)
                .with_targets([
                    ("layer", tracing::level_filters::LevelFilter::OFF),
                    ("store", tracing::level_filters::LevelFilter::OFF),
                    ("ws", tracing::level_filters::LevelFilter::OFF),
                ]),
        );
        tracing::subscriber::set_global_default(subscriber).map_err(Into::into)
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const X_USER: hyper::header::HeaderName = hyper::header::HeaderName::from_static("x-user");

fn main() -> std::process::ExitCode {
    let (verbosity, port, threads, create, db_root, users) = args::parse().decompose();

    if let Err(err) = setup_tracing(verbosity) {
        eprintln!("{err}");
        return std::process::ExitCode::FAILURE;
    }

    tracing::info!(
        verbosity = %verbosity.level,
        spans = %verbosity.include_spans,
        port = port,
        threads = %threads,
        create = %create,
        db_root = %db_root.display(),
        users = ?users,
        "Configuration loaded"
    );

    if users.is_empty() {
        tracing::error!("No users provided");
        return std::process::ExitCode::FAILURE;
    }

    if create && !service::create_users(&db_root, &users) {
        return std::process::ExitCode::FAILURE;
    }

    let runtime = match boile_rs::rt::runtime(
        #[cfg(feature = "threads")]
        threads,
    )
    .enable_all()
    .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "Failed to build the async runtime");
            return std::process::ExitCode::FAILURE;
        }
    };

    runtime.block_on(async_main(port, db_root, users))
}

async fn async_main(
    port: u16,
    db_root: std::path::PathBuf,
    users: std::collections::HashSet<String>,
) -> std::process::ExitCode {
    let services = match service::new(db_root, users).await {
        Ok(services) => services,
        Err(error) => {
            tracing::error!(%error, "Failed to create the store service");
            return std::process::ExitCode::FAILURE;
        }
    };

    let router = router::build()
        .layer(layer::Auth::wrap(services))
        .layer(layer::Logger);

    let addr = ([0, 0, 0, 0], port).into();

    tracing::info!(%addr, "Binding to address");

    let server = hyper::Server::bind(&addr).serve(router.into_make_service());

    let server = match boile_rs::rt::Shutdown::new() {
        Ok(shutdown) => server.with_graceful_shutdown(shutdown),
        Err(error) => {
            tracing::error!(%error, "Failed to create shutdown hook");
            return std::process::ExitCode::FAILURE;
        }
    };

    let start = std::time::Instant::now();

    if let Err(error) = server.await {
        tracing::error!(%error, duration = ?start.elapsed(), "Server execution aborted");
        std::process::ExitCode::FAILURE
    } else {
        tracing::info!(duration = ?start.elapsed(), "Server gracefully shutdown");
        std::process::ExitCode::SUCCESS
    }
}
