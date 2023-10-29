// use crate::check_async as check;
// use crate::{client, helper, test_utils};

use crate::{client, utils};

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
    let db_root = utils::TestPath::new();

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
    let mut port = 27720;
    loop {
        assert!(port >= 27720, "Could not find available port above 27720");
        if port_unused(port) {
            return port;
        }
        port = port.wrapping_add(1);
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
        .arg(utils::USER)
        .arg("-U")
        .arg(utils::EMPTY_USER)
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
        const SKULL: &str = r#"{"name":"skull$","color":$,"icon":"icon$","price":0.$}"#;
        const QUICK: &str = r#"{"skull":$,"amount":$.0}"#;
        const OCCURRENCE: &str = r#"{"skull":$,"amount":$.0,"millis":$}"#;

        self.insert_items("/skull", SKULL).await;
        self.check_items("/skull", SKULL).await;

        self.insert_items("/quick", QUICK).await;
        self.check_items("/quick", QUICK).await;

        self.insert_occurrences("/occurrence", OCCURRENCE).await;
        self.check_items("/occurrence", OCCURRENCE).await;
    }

    async fn insert_items(&self, path: &'static str, template: &'static str) {
        for i in 1..=3 {
            let response = self
                .client
                .post(path, template.replace('$', &i.to_string()))
                .await;

            let body = utils::extract_body(response).await;

            assert_eq!(body, "\"created\"");
        }
    }

    async fn insert_occurrences(&self, path: &'static str, template: &'static str) {
        let items = (1..=3)
            .map(|i| template.replace('$', &i.to_string()))
            .collect::<Vec<_>>()
            .join(",");
        let payload = format!(r#"{{"items":[{items}]}}"#);

        let response = self.client.post(path, payload).await;

        let body = utils::extract_body(response).await;
        assert_eq!(body, "\"created\"");
    }

    async fn check_items(&self, path: &'static str, template: &'static str) {
        // let response = self.client.get(path).await;
        //
        // let template = template.replace('{', r#"{"id":$,"#);
        //
        // let expected_body = if path == "/occurrence" {
        //     format!(
        //         "[{},{},{}]",
        //         template.replace('$', "3"),
        //         template.replace('$', "2"),
        //         template.replace('$', "1")
        //     )
        // } else {
        //     format!(
        //         "[{},{},{}]",
        //         template.replace('$', "1"),
        //         template.replace('$', "2"),
        //         template.replace('$', "3"),
        //     )
        // };
        //
        // check!(helper::eq(
        //     response,
        //     hyper::StatusCode::OK,
        //     helper::LastModified::Eq(last_modified),
        //     expected_body,
        // ));
    }
}
