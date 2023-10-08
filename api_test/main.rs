mod client;
mod helper;
mod server;
mod tests;

fn main() -> std::process::ExitCode {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let errors = run(&runtime);
    println!();

    if errors.is_empty() {
        std::process::ExitCode::SUCCESS
    } else {
        println!("failures:");
        for error in errors {
            println!("    {error}");
        }
        println!();
        std::process::ExitCode::FAILURE
    }
}

fn run(runtime: &tokio::runtime::Runtime) -> Vec<&'static str> {
    let server = runtime.block_on(server::start());

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
