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
        socket = %args.socket,
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
        socket = %args.socket,
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
        args.threads,
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

    runtime.block_on(async_main(args.socket, args.db, args.users))
}

async fn async_main(
    socket: args::Socket,
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

    let shutdown = match boile_rs::rt::Shutdown::new() {
        Ok(shutdown) => shutdown,
        Err(error) => {
            tracing::error!(%error, "Failed to create shutdown hook");
            return std::process::ExitCode::FAILURE;
        }
    };

    let start = std::time::Instant::now();

    let result = match socket {
        args::Socket::Port(port) => match make_tcp_listener(port).await {
            Some(listener) => {
                axum::serve(listener, router.into_make_service())
                    .with_graceful_shutdown(shutdown)
                    .await
            }
            None => return std::process::ExitCode::FAILURE,
        },
        args::Socket::Unix(path) => match make_unix_listener(path) {
            Some(listener) => {
                axum::serve(listener, router.into_make_service())
                    .with_graceful_shutdown(shutdown)
                    .await
            }
            None => return std::process::ExitCode::FAILURE,
        },
    };

    if let Err(error) = result {
        tracing::error!(%error, duration = ?start.elapsed(), "Server execution aborted");
        std::process::ExitCode::FAILURE
    } else {
        tracing::info!(duration = ?start.elapsed(), "Server gracefully shutdown");
        std::process::ExitCode::SUCCESS
    }
}

async fn make_tcp_listener(port: u16) -> Option<tokio::net::TcpListener> {
    let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, port);
    tracing::info!(%addr, "Binding to address");

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => Some(listener),
        Err(error) => {
            tracing::error!(%error, "Failed to bind to address");
            None
        }
    }
}

fn make_unix_listener(path: std::path::PathBuf) -> Option<tokio::net::UnixListener> {
    tracing::info!(path = %path.display(), "Binding to unix socket");

    match tokio::net::UnixListener::bind(path) {
        Ok(listener) => Some(listener),
        Err(error) => {
            tracing::error!(%error, "Failed to bind to unix socket");
            None
        }
    }
}
