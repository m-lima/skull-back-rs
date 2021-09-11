mod middleware;
mod options;
mod router;
mod store;

fn init_logger() {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_str("%Y-%m-%dT%H:%M:%SZ")
        .build();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");
}

fn main() {
    let options = options::parse();
    init_logger();

    if options.threads > 0 {
        let threads = usize::from(options.threads);
        log::info!("Core threads set to {}", options.threads);
        gotham::start_with_num_threads(
            format!("0.0.0.0:{}", options.port),
            router::route(options),
            threads,
        );
    } else {
        log::info!("Core threads set to automatic");
        gotham::start(format!("0.0.0.0:{}", options.port), router::route(options));
    }
}

#[cfg(test)]
mod test {
    use super::options;
    use gotham::test::TestServer;

    fn options() -> options::Options {
        options::Options {
            port: 0,
            threads: 0,
            cors: None,
            store_path: None,
            web_path: None,
        }
    }

    #[test]
    fn extractors() {
        let test_server = TestServer::new(super::router::route(options())).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/Skull/3")
            .perform()
            .unwrap();

        assert_eq!(response.status(), gotham::hyper::StatusCode::OK);
        assert_eq!(
            response.read_body().unwrap(),
            String::from("Id: 3, Store: Skull").as_bytes()
        );
    }
}
