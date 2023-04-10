use gotham::hyper;
use pwner::Spawner;
use test_utils::check_async as check;

use super::{eq, LastModified};

static SERVER: tokio::sync::OnceCell<Process> = tokio::sync::OnceCell::const_new();

// pub enum Type {
//     Memory,
//     File(std::path::PathBuf),
//     Db(std::path::PathBuf),
// }

struct Process {
    uri: String,
    #[allow(dead_code)]
    process: pwner::process::Duplex,
}

pub struct Server {
    client: hyper::Client<hyper::client::HttpConnector>,
    uri: &'static str,
}

impl Server {
    pub async fn instance() -> Server {
        let process = SERVER
            .get_or_init(|| async {
                let port = random_port();
                let process = server(port);
                let uri = format!("localhost:{port}");

                wait_for_server(port).await;
                Populator::new(&uri).populate().await;

                Process { uri, process }
            })
            .await;

        Server {
            uri: &process.uri,
            client: hyper::Client::new(),
        }
    }

    pub async fn get(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = http::get(self.uri, path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn get_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = http::get(self.uri, path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn post(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        if path_and_query.as_ref().starts_with("/skull") {
            eprintln!("Warning: attempting to post `/skull`");
        }
        let request = http::post(self.uri, path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn put(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        if path_and_query.as_ref().starts_with("/skull") {
            eprintln!("Warning: attempting to put `/skull`");
        }
        let request = http::put(self.uri, path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn put_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        if path_and_query.as_ref().starts_with("/skull") {
            eprintln!("Warning: attempting to put `/skull`");
        }
        let mut request = http::put(self.uri, path_and_query, body);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        if path_and_query.as_ref().starts_with("/skull") {
            eprintln!("Warning: attempting to delete `/skull`");
        }
        let request = http::delete(self.uri, path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        if path_and_query.as_ref().starts_with("/skull") {
            eprintln!("Warning: attempting to delete `/skull`");
        }
        let mut request = http::delete(self.uri, path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn head(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = http::head(self.uri, path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn head_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = http::head(self.uri, path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn last_modified(&self, path: impl AsRef<str>) -> u64 {
        http::last_modified(&self.client, self.uri, path).await
    }
}

struct Populator<'a> {
    uri: &'a str,
    client: hyper::Client<hyper::client::HttpConnector>,
}

impl Populator<'_> {
    fn new(uri: &str) -> Populator<'_> {
        Populator {
            uri,
            client: hyper::Client::new(),
        }
    }

    async fn populate(&self) {
        const SKULL: &str = r#"{"name":"skull$","color":"color$","icon":"icon$","unitPrice":0.$}"#;
        const QUICK: &str = r#"{"skull":$,"amount":$.0}"#;
        const OCCURRENCE: &str = r#"{"skull":$,"amount":$.0,"millis":$}"#;

        let mut modified = http::last_modified(&self.client, self.uri, "/skull").await;
        modified = self.insert_items("/skull", SKULL, modified).await;
        self.check_items("/skull", SKULL, modified).await;

        let mut modified = http::last_modified(&self.client, self.uri, "/quick").await;
        modified = self.insert_items("/quick", QUICK, modified).await;
        self.check_items("/quick", QUICK, modified).await;

        let mut modified = http::last_modified(&self.client, self.uri, "/occurrence").await;
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

            let request = http::post(self.uri, path, template.replace('$', &i.to_string()));
            let response = self.client.request(request).await.unwrap();

            let expected_last_modified = if now.elapsed().as_millis() > 0 {
                LastModified::Gt(last_modified)
            } else {
                LastModified::Ge(last_modified)
            };

            last_modified = check!(eq(
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
        let request = http::get(self.uri, path);
        let response = self.client.request(request).await.unwrap();

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

        check!(eq(
            response,
            hyper::StatusCode::OK,
            LastModified::Eq(last_modified),
            expected_body,
        ));
    }
}

mod http {
    use super::super::extract_last_modified;

    type Client = hyper::Client<hyper::client::HttpConnector>;

    pub fn get(uri: &str, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        request(uri, path_and_query)
            .method(hyper::Method::GET)
            .header(test_utils::http::USER_HEADER, test_utils::USER)
            .body(hyper::Body::empty())
            .unwrap()
    }

    pub fn post(
        uri: &str,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
        request(uri, path_and_query)
            .method(hyper::Method::POST)
            .header(test_utils::http::USER_HEADER, test_utils::USER)
            .body(body.into())
            .unwrap()
    }

    pub fn put(
        uri: &str,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
        request(uri, path_and_query)
            .method(hyper::Method::PUT)
            .header(test_utils::http::USER_HEADER, test_utils::USER)
            .header(hyper::header::IF_UNMODIFIED_SINCE, millis_in_future())
            .body(body.into())
            .unwrap()
    }

    pub fn delete(uri: &str, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        request(uri, path_and_query)
            .method(hyper::Method::DELETE)
            .header(test_utils::http::USER_HEADER, test_utils::USER)
            .header(hyper::header::IF_UNMODIFIED_SINCE, millis_in_future())
            .body(hyper::Body::empty())
            .unwrap()
    }

    pub fn head(uri: &str, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        request(uri, path_and_query)
            .method(hyper::Method::HEAD)
            .header(test_utils::http::USER_HEADER, test_utils::USER)
            .body(hyper::Body::empty())
            .unwrap()
    }

    pub async fn last_modified(client: &Client, uri: &str, path: impl AsRef<str>) -> u64 {
        let request = head(uri, path);
        let response = client.request(request).await.unwrap();
        extract_last_modified(&response).unwrap()
    }

    fn request(uri: &str, path_and_query: impl AsRef<str>) -> hyper::http::request::Builder {
        let uri = hyper::Uri::builder()
            .scheme("http")
            .authority(uri)
            .path_and_query(path_and_query.as_ref())
            .build()
            .unwrap();
        hyper::Request::builder().uri(uri)
    }

    fn millis_in_future() -> hyper::http::HeaderValue {
        hyper::header::HeaderValue::from_str(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .saturating_add(std::time::Duration::from_secs(10))
                .as_millis()
                .to_string()
                .as_str(),
        )
        .unwrap()
    }
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
    std::process::Command::new(env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME"))))
        .arg("-t")
        .arg("1")
        .arg("-u")
        .arg(test_utils::USER)
        .arg("-u")
        .arg(super::EMPTY_USER)
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
