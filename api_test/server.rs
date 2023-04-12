use test_utils::check_async as check;

use crate::helper;

pub struct Server {
    uri: String,
    _process: pwner::process::Simplex,
}

impl Server {
    pub fn uri(&self, path_and_query: impl AsRef<str>) -> hyper::Uri {
        hyper::Uri::builder()
            .scheme("http")
            .authority(self.uri.as_str())
            .path_and_query(path_and_query.as_ref())
            .build()
            .unwrap()
    }
}

pub async fn start() -> Server {
    let port = random_port();

    let server = Server {
        uri: format!("localhost:{port}"),
        _process: server(port).decompose().0,
    };

    wait_for_server(port).await;

    Populator::new(&server).populate().await;

    server
}

fn random_port() -> u16 {
    static PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(27720);

    loop {
        let port = PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(port >= 27720, "Could not find available port above 27720");
        if port_unused(port) {
            return port;
        }
    }
}

fn port_unused(port: u16) -> bool {
    std::net::TcpListener::bind(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::UNSPECIFIED,
        port,
    ))
    .ok()
    .and_then(|l| l.local_addr().ok())
    .map(|l| l.port())
    .filter(|p| *p == port)
    .is_some()
}

fn server(port: u16) -> pwner::process::Duplex {
    use pwner::Spawner;

    std::process::Command::new(env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME"))))
        .arg("-t")
        .arg("1")
        .arg("-u")
        .arg(test_utils::USER)
        .arg("-u")
        .arg(helper::EMPTY_USER)
        .arg("-p")
        .arg(format!("{port}"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .spawn_owned()
        .unwrap()
}

async fn wait_for_server(port: u16) {
    let now = std::time::Instant::now();
    while port_unused(port) {
        assert!(
            now.elapsed() < std::time::Duration::from_secs(10),
            "Timeout waiting for the server to start"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

struct Populator<'a> {
    client: crate::client::Client<'a>,
}

impl Populator<'_> {
    fn new<'a>(server: &'a Server) -> Populator<'a> {
        Populator {
            client: server.into(),
        }
    }

    async fn populate(&self) {
        const SKULL: &str = r#"{"name":"skull$","color":"color$","icon":"icon$","unitPrice":0.$}"#;
        const QUICK: &str = r#"{"skull":$,"amount":$.0}"#;
        const OCCURRENCE: &str = r#"{"skull":$,"amount":$.0,"millis":$}"#;

        let mut modified = self.client.last_modified("/skull").await;
        modified = self.insert_items("/skull", SKULL, modified).await;
        self.check_items("/skull", SKULL, modified).await;

        let mut modified = self.client.last_modified("/quick").await;
        modified = self.insert_items("/quick", QUICK, modified).await;
        self.check_items("/quick", QUICK, modified).await;

        let mut modified = self.client.last_modified("/occurrence").await;
        modified = self.insert_items("/occurrence", OCCURRENCE, modified).await;
        self.check_items("/occurrence", OCCURRENCE, modified).await;
    }

    async fn insert_items(
        &self,
        path: &'static str,
        template: &'static str,
        mut last_modified: u64,
    ) -> u64 {
        for i in 1..=3 {
            let now = std::time::Instant::now();

            let response = self
                .client
                .post(path, template.replace('$', &i.to_string()))
                .await;

            let expected_last_modified = if now.elapsed().as_millis() > 0 {
                helper::LastModified::Gt(last_modified)
            } else {
                helper::LastModified::Ge(last_modified)
            };

            last_modified = check!(helper::eq(
                response,
                hyper::StatusCode::CREATED,
                expected_last_modified,
                i.to_string(),
            ))
            .unwrap();
        }
        last_modified
    }

    async fn check_items(&self, path: &'static str, template: &'static str, last_modified: u64) {
        let response = self.client.get(path).await;

        let template = template.replace('{', r#"{"id":$,"#);

        let expected_body = if path == "/occurrence" {
            format!(
                "[{},{},{}]",
                template.replace('$', "3"),
                template.replace('$', "2"),
                template.replace('$', "1")
            )
        } else {
            format!(
                "[{},{},{}]",
                template.replace('$', "1"),
                template.replace('$', "2"),
                template.replace('$', "3"),
            )
        };

        check!(helper::eq(
            response,
            hyper::StatusCode::OK,
            helper::LastModified::Eq(last_modified),
            expected_body,
        ));
    }
}
