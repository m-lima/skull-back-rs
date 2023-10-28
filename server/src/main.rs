mod args;
mod layer;
mod rest;
mod router;
mod service;
mod ws;

#[allow(clippy::declare_interior_mutable_const)]
const X_USER: hyper::header::HeaderName = hyper::header::HeaderName::from_static("x-user");

fn main() -> std::process::ExitCode {
    let (verbosity, port, threads, create, db_root, users) = args::parse().decompose();

    if let Err(err) = boile_rs::log::setup(verbosity) {
        eprintln!("{err}");
        return std::process::ExitCode::FAILURE;
    }

    tracing::info!(
        verbosity = %verbosity,
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

    let router = axum::Router::<(), hyper::Body>::new()
        .nest("/", rest::build())
        .nest("/ws", ws::build())
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
