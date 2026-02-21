#![allow(dead_code, unused_variables)]

mod client;
mod server;
mod utils;
// mod test_utils;
// mod tests;

fn main() -> std::process::ExitCode {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let start = std::time::Instant::now();
    let (count, errors) = run(&runtime);
    let elapsed = start.elapsed();
    println!();

    if errors.is_empty() {
        println!("test result: [32mok[m. {count} passed; 0 failed; finished in {elapsed:?}");
        std::process::ExitCode::SUCCESS
    } else {
        let ok = count - errors.len();
        println!(
            "test result: [32mok[m. {ok} passed; {} failed; finished in {elapsed:?}",
            errors.len()
        );
        println!();
        println!("failures:");
        for error in errors {
            println!("    {error}");
        }
        println!();
        std::process::ExitCode::FAILURE
    }
}

// TODO: These tests
fn run(runtime: &tokio::runtime::Runtime) -> (usize, Vec<&'static str>) {
    let server = runtime.block_on(server::start());
    (0, Vec::new())

    // let tests = tests::test(runtime, &server);
    // let count = tests.len();
    //
    // let errors = if tests.is_empty() {
    //     Vec::new()
    // } else {
    //     println!("running {} tests", tests.len());
    //     tests
    //         .into_iter()
    //         .filter_map(|(name, test)| {
    //             if test.is_ok() {
    //                 println!("test {name} ... [32mok[m");
    //                 None
    //             } else {
    //                 println!("test {name} ... [31mFAILED[m");
    //                 Some(name)
    //             }
    //         })
    //         .collect()
    // };
    //
    // (count, errors)
}
