mod args;
mod layer;
mod router;
mod service;
mod ws;

fn setup_tracing(
    verbosity: args::Verbosity,
) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber =
        tracing_subscriber::registry().with(boile_rs::log::tracing::layer(boile_rs::log::Stdout));

    if verbosity.include_spans {
        let subscriber = subscriber.with(::tracing::level_filters::LevelFilter::from_level(
            verbosity.level,
        ));
        tracing::subscriber::set_global_default(subscriber)
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
        tracing::subscriber::set_global_default(subscriber)
    }
}

fn main() -> std::process::ExitCode {
    let args = args::parse();

    if let Err(err) = setup_tracing(args.verbosity) {
        eprintln!("{err}");
        return std::process::ExitCode::FAILURE;
    }

    #[cfg(feature = "threads")]
    tracing::info!(
        verbosity = %args.verbosity.level,
        spans = %args.verbosity.include_spans,
        port = args.port,
        threads = %args.threads,
        create = %args.create,
        db_root = %args.db.display(),
        users = ?args.users,
        "Configuration loaded"
    );
    #[cfg(not(feature = "threads"))]
    tracing::info!(
        verbosity = %args.verbosity.level,
        spans = %args.verbosity.include_spans,
        port = args.port,
        threads = %"single",
        create = %args.create,
        db_root = %args.db.display(),
        users = ?args.users,
        "Configuration loaded"
    );

    if args.users.is_empty() {
        tracing::error!("No users provided");
        return std::process::ExitCode::FAILURE;
    }

    if args.create && !service::create_users(&args.db, &args.users) {
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

    runtime.block_on(async_main(args.port, args.db, args.users))
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

    let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, port);
    tracing::info!(%addr, "Binding to address");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, "Failed to bind to address");
            return std::process::ExitCode::FAILURE;
        }
    };

    let server = match boile_rs::rt::Shutdown::new() {
        Ok(shutdown) => {
            axum::serve(listener, router.into_make_service()).with_graceful_shutdown(shutdown)
        }
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
