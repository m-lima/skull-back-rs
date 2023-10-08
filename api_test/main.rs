mod client;
mod helper;
mod server;
mod tests;

fn main() -> std::process::ExitCode {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let errors = run(&runtime, server::Mode::Memory);
    println!();
    if !errors.is_empty() {
        println!("failures:");
        for error in errors {
            println!("    {error}");
        }
        println!();
        return std::process::ExitCode::FAILURE;
    }

    let errors = run(&runtime, server::Mode::File(test_utils::TestPath::new()));
    println!();
    if !errors.is_empty() {
        println!("failures:");
        for error in errors {
            println!("    {error}");
        }
        println!();
        return std::process::ExitCode::FAILURE;
    }

    let errors = run(&runtime, server::Mode::Db(test_utils::TestPath::new()));
    println!();
    if !errors.is_empty() {
        println!("failures:");
        for error in errors {
            println!("    {error}");
        }
        println!();
        return std::process::ExitCode::FAILURE;
    }

    std::process::ExitCode::SUCCESS
}

fn run(runtime: &tokio::runtime::Runtime, mode: server::Mode) -> Vec<String> {
    let server = runtime.block_on(server::start(mode));

    let tests = tests::test(runtime, &server);
    if tests.is_empty() {
        Vec::new()
    } else {
        println!("running {} tests", tests.len());
        tests
            .into_iter()
            .filter_map(|(name, test)| {
                if test.is_ok() {
                    println!("test {name} ... [32mok[m");
                    None
                } else {
                    println!("test {name} ... [31mFAILED[m");
                    Some(name)
                }
            })
            .collect()
    }
}
