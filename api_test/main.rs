mod client;
mod helper;
mod in_memory;
mod server;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(future)
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let server = runtime.block_on(server::start());

    match std::thread::scope(|s| {
        s.spawn(|| {
            // std::panic::catch_unwind(|| {
            runtime.block_on(in_memory::list(client::Client::from(&server)))
        })
    })
    .join()
    {
        Ok(()) => println!("Ok"),
        Err(err) => eprintln!("{err:?}"),
    }
}
