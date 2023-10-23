#[cfg(feature = "threads")]
#[must_use]
pub fn runtime(threads: Threads) -> tokio::io::Result<tokio::runtime::Runtime> {
    tracing::info!(%threads, "Building tokio runtime");
    match threads {
        Threads::Single => tokio::runtime::Builder::new_current_thread(),
        Threads::Auto => tokio::runtime::Builder::new_multi_thread(),
        Threads::Multi(threads::Count(count)) => {
            let mut rt = tokio::runtime::Builder::new_multi_thread();
            rt.worker_threads(usize::from(count));
            rt
        }
    }
    .enable_all()
    .build()
}

#[cfg(not(feature = "threads"))]
#[must_use]
pub fn runtime() -> tokio::io::Result<tokio::runtime::Runtime> {
    tracing::info!(threads = %"Single", "Building tokio runtime");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
}
