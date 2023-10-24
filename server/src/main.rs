mod args;
mod auth;
mod error;
mod logger;
mod router;
mod runtime;
mod service;
mod ws;

fn main() -> std::process::ExitCode {
    let args = args::parse();

    if let Err(err) = boile_rs::log::setup(args.verbosity()) {
        eprintln!("{err}");
        return std::process::ExitCode::FAILURE;
    }

    let port = args.port();
    let threads = args.threads();
    let create = args.create();
    let (users, db_root) = args.db();

    tracing::info!(
        port = port,
        threads = ?threads,
        create = %create,
        db_root = %db_root.display(),
        users = ?users,
        "Configuration loaded"
    );

    if users.is_empty() {
        tracing::error!("No users provided");
        return std::process::ExitCode::FAILURE;
    }

    let runtime = match runtime::runtime(
        #[cfg(feature = "threads")]
        threads,
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "Failed to build the async runtime");
            return std::process::ExitCode::FAILURE;
        }
    };

    if create {
        for user in &users {
            let path = db_root.join(user);
            if !path.exists() {
                tracing::info!(db = %path.display(), "Creating database");
                if let Err(error) = std::fs::write(&path, []) {
                    tracing::error!(db = %path.display(), %error, "Unable to create database");
                    return std::process::ExitCode::FAILURE;
                }
            }
        }
    }

    runtime.block_on(async_main(port));

    std::process::ExitCode::SUCCESS
}

async fn async_main(port: u16) {}

// auth
// - Rejects
// - ws/rest splitter
//     - ws
//         - Creates connection
//             - ws
//     - rest
//         - Root
//             - logger
//         - Path
//             - create root request
//                 - logger
//
// ws
// - logger
//
// logger
// - take duration
//     - call handler
//         - response of handler should have "action" and Result<types::Response>
//             - Make json
//                 - Log duration, size, action
//                     - Respond
