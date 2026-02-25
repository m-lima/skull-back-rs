#![deny(warnings, clippy::pedantic, clippy::all, rust_2018_idioms)]

mod args;
mod cli;
mod constant;
mod handler;
mod request;
mod secret;

macro_rules! pass {
    ($fallible: expr) => {
        match $fallible {
            Ok(ok) => ok,
            Err(err) => {
                if err.canceled() {
                    err.post();
                    return std::process::ExitCode::SUCCESS;
                }
                println!("[31mError:[m {err}");
                err.post();
                return std::process::ExitCode::FAILURE;
            }
        }
    };
}

trait Cancelable: PostAction {
    fn canceled(&self) -> bool {
        false
    }
}

trait PostAction: Sized {
    fn post(self) {}
}

fn main() -> std::process::ExitCode {
    let command = pass!(args::parse());
    let secret = pass!(secret::Secret::new());
    let request = pass!(request::Request::new(secret));
    let handler = handler::Handler::new(request);
    let runtime = pass!(runtime::build());

    runtime.block_on(async_main(command, handler))
}

async fn async_main(command: args::Command, handler: handler::Handler) -> std::process::ExitCode {
    pass!(match command {
        args::Command::List => handler.list().await,
        args::Command::Update => handler.update().await,
        args::Command::Register(args) => handler.register(args).await,
        args::Command::Dump => handler.dump().await,
        args::Command::Plot(args) => handler.plot(args).await,
    });

    handler.post();
    std::process::ExitCode::SUCCESS
}

mod runtime {
    #[derive(Debug, thiserror::Error)]
    #[error("Failed to build tokio runtime: {0}")]
    pub struct Error(std::io::Error);

    impl crate::PostAction for Error {}
    impl crate::Cancelable for Error {}

    pub fn build() -> Result<tokio::runtime::Runtime, Error> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(Error)
    }
}
