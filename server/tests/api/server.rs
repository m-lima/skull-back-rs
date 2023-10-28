use crate::check_async as check;
use crate::{client, helper, test_utils};

pub struct Server {
    uri: std::sync::Arc<String>,
    _process: pwner::process::Simplex,
}

impl Server {
    pub fn uri(&self) -> std::sync::Arc<String> {
        self.uri.clone()
    }

    pub fn client(&self) -> client::Client {
        self.into()
    }
}

pub async fn start() -> Server {
    let port = random_port();
    let db_root = test_utils::TestPath::new();

    let (process, mut output) = server(port, &db_root).decompose();
    let server = Server {
        uri: std::sync::Arc::new(format!("localhost:{port}")),
        _process: process,
    };

    if !wait_for_server(port).await {
        let mut output_string = String::new();
        std::io::Read::read_to_string(&mut output, &mut output_string).unwrap();
        eprintln!("Server stdout:");
        eprintln!("{output_string}");
        let mut output_string = String::new();
        output.read_from(pwner::process::ReadSource::Stderr);
        std::io::Read::read_to_string(&mut output, &mut output_string).unwrap();
        eprintln!("Server stderr:");
        eprintln!("{output_string}");
        panic!("Timeout waiting for the server to start");
    }

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

fn server(port: u16, db_root: &std::path::Path) -> pwner::process::Duplex {
    use pwner::Spawner;

    std::process::Command::new(env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME"))))
        .arg("-c")
        .arg("-U")
        .arg(test_utils::USER)
        .arg("-U")
        .arg(helper::EMPTY_USER)
        .arg("-p")
        .arg(format!("{port}"))
        .arg(db_root.to_str().unwrap())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .spawn_owned()
        .unwrap()
}

#[must_use]
async fn wait_for_server(port: u16) -> bool {
    let now = std::time::Instant::now();
    while port_unused(port) {
        if now.elapsed() > std::time::Duration::from_secs(10) {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    true
}

struct Populator {
    client: crate::client::Client,
}

impl Populator {
    fn new(server: &Server) -> Populator {
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
