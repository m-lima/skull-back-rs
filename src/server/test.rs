use crate::{check, test_util::Assertion};

const USER: &str = "bloink";

#[derive(Copy, Clone)]
struct CopiablePath {
    data: [u8; 1024],
    len: usize,
}

impl CopiablePath {
    fn new(path: &std::path::Path) -> Self {
        let path = path.to_str().unwrap().as_bytes();
        let len = path.len();
        let mut data = [0_u8; 1024];
        data[..len].copy_from_slice(path);

        Self { data, len }
    }

    fn into_path(self) -> std::path::PathBuf {
        String::from_utf8(Vec::from(&self.data[..self.len]))
            .unwrap()
            .into()
    }
}

struct TestServer {
    server: gotham::test::TestServer,
    path: std::path::PathBuf,
}

impl TestServer {
    fn new() -> Self {
        let name = format!("{:016x}", rand::random::<u64>());
        let path = std::env::temp_dir().join("skull-test");
        if path.exists() {
            assert!(path.is_dir(), "Cannot use {} as test path", path.display());
        } else {
            std::fs::create_dir(&path).unwrap();
        }
        let path = path.join(name);
        assert!(
            !path.exists(),
            "Cannot use {} as test path as it already exists",
            path.display()
        );
        std::fs::create_dir(&path).unwrap();
        let copiable_path = CopiablePath::new(&path);

        let server = gotham::test::TestServer::new(move || {
            super::route(crate::options::Options {
                port: 0,
                threads: 0,
                cors: None,
                db_path: Some(copiable_path.into_path()),
                store_path: None,
                web_path: None,
                users: vec![String::from(USER)],
            })
        })
        .unwrap();

        let server = Self { server, path };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(server.initialize_db())
            .unwrap();

        server
    }

    async fn initialize_db(&self) -> Result<(), sqlx::Error> {
        let path = self.path.join(USER);
        std::fs::File::create(&path).unwrap();

        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        sqlx::migrate!().run(&pool).await?;

        Ok(())
    }

    fn last_modified(&self) -> u64 {
        self.client()
            .get("http://localhost/skull")
            .with_header(
                super::mapper::request::USER_HEADER,
                gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
            )
            .perform()
            .unwrap()
            .headers()
            .get(gotham::hyper::header::LAST_MODIFIED)
            .unwrap()
            .to_str()
            .map(str::parse)
            .unwrap()
            .unwrap()
    }

    fn populate(&self) -> u64 {
        assert_eq!(
            self.client()
                .post(
                    "http://localhost/skull",
                    r#"{
                        "name": "skull1",
                        "color": "color1",
                        "icon": "icon1",
                        "unitPrice": 0.1
                    }"#,
                    mime::APPLICATION_JSON,
                )
                .with_header(
                    super::mapper::request::USER_HEADER,
                    gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
                )
                .perform()
                .unwrap()
                .status(),
            201
        );

        assert_eq!(
            self.client()
                .post(
                    "http://localhost/skull",
                    r#"{
                        "name": "skull2",
                        "color": "color2",
                        "icon": "icon2",
                        "unitPrice": 0.2
                    }"#,
                    mime::APPLICATION_JSON,
                )
                .with_header(
                    super::mapper::request::USER_HEADER,
                    gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
                )
                .perform()
                .unwrap()
                .status(),
            201
        );

        assert_eq!(
            self.client()
                .post(
                    "http://localhost/skull",
                    r#"{
                        "name": "skull3",
                        "color": "color3",
                        "icon": "icon3",
                        "unitPrice": 0.3
                    }"#,
                    mime::APPLICATION_JSON,
                )
                .with_header(
                    super::mapper::request::USER_HEADER,
                    gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
                )
                .perform()
                .unwrap()
                .status(),
            201
        );

        let response = self
            .client()
            .get("http://localhost/skull")
            .with_header(
                super::mapper::request::USER_HEADER,
                gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
            )
            .perform()
            .unwrap();

        let last_modified = response
            .headers()
            .get(gotham::hyper::header::LAST_MODIFIED)
            .unwrap()
            .to_str()
            .map(str::parse)
            .unwrap()
            .unwrap();

        assert_eq!(
            serde_json::from_str::<Vec<crate::store::Skull>>(
                response.read_utf8_body().unwrap().as_str(),
            )
            .unwrap()
            .len(),
            3
        );

        last_modified
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.path));
    }
}

impl std::ops::Deref for TestServer {
    type Target = gotham::test::TestServer;

    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

fn response_eq(
    response: gotham::test::TestResponse,
    expected_status: u16,
    expected_last_modified: LastModified,
    expected_body: &str,
) -> Assertion<Option<u64>> {
    if response.status() != expected_status {
        return Assertion::err_ne("Status code mismatch", response.status(), expected_status);
    }

    let last_modified = response
        .headers()
        .get(gotham::hyper::header::LAST_MODIFIED)
        .map(|h| h.to_str().unwrap().parse().unwrap());
    if last_modified != expected_last_modified {
        return Assertion::err_ne(
            "Last modified mismatch",
            last_modified,
            expected_last_modified,
        );
    }

    let body = response.read_utf8_body().unwrap();
    if body != expected_body {
        return Assertion::err_ne("Body mismatch", body, expected_body);
    }

    Assertion::Ok(last_modified)
}

#[derive(Debug, Copy, Clone)]
enum LastModified {
    None,
    Eq(u64),
    Gt(u64),
}

impl PartialEq<LastModified> for Option<u64> {
    fn eq(&self, other: &LastModified) -> bool {
        match other {
            LastModified::None => self.is_none(),
            LastModified::Eq(o) => self.map_or(false, |s| s == *o),
            LastModified::Gt(o) => self.map_or(false, |s| s > *o),
        }
    }
}

#[test]
fn forbidden() {
    let server = TestServer::new();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str("unknown").unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 403, LastModified::None, ""));
}

#[test]
fn empty() {
    let server = TestServer::new();
    let last_modified = server.last_modified();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        "[]"
    ));
}

#[test]
fn bad_request() {
    let server = TestServer::new();

    let response = server
        .client()
        .post(
            "http://localhost/occurrence",
            r#"{
                "skul": 666,
                "amount": 1,
                "millis": 4000
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 400, LastModified::None, ""));
}

#[test]
fn list() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn list_limited() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .get("http://localhost/skull?limit=1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3}]"#,
    ));

    let response = server
        .client()
        .get("http://localhost/skull?limit=0")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[]"#
    ));
}

#[test]
fn read() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .get("http://localhost/skull/2")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2}"#,
    ));
}

#[test]
fn read_not_found() {
    let server = TestServer::new();
    server.populate();

    let response = server
        .client()
        .get("http://localhost/skull/666")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 404, LastModified::None, ""));
}

#[test]
fn update() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .put(
            "http://localhost/skull/2",
            r#"{
                "name": "skull4",
                "color": "",
                "icon": "",
                "unitPrice": 0.4
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    let last_modified = check!(response_eq(
        response,
        200,
        LastModified::Gt(last_modified),
        r#"{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2}"#,
    ))
    .unwrap();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull4","color":"","icon":"","unitPrice":0.4},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn update_not_found() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .put(
            "http://localhost/skull/666",
            r#"{
                "name": "skull4",
                "color": "",
                "icon": "",
                "unitPrice": 0.4
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 404, LastModified::None, ""));

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn delete() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .delete("http://localhost/skull/2")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    let last_modified = check!(response_eq(
        response,
        200,
        LastModified::Gt(last_modified),
        r#"{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2}"#,
    ))
    .unwrap();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn delete_not_found() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .delete("http://localhost/skull/666")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 404, LastModified::None, ""));

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn no_modified_since() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .put(
            "http://localhost/skull/1",
            r#"{
                "name": "skull4",
                "color": "",
                "icon": "",
                "unitPrice": 0.4
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 412, LastModified::None, ""));

    let response = server
        .client()
        .delete("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 412, LastModified::None, ""));

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn out_of_sync() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .put(
            "http://localhost/skull/1",
            r#"{
                "name": "skull4",
                "color": "",
                "icon": "",
                "unitPrice": 0.4
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str("100").unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 412, LastModified::None, ""));

    let response = server
        .client()
        .delete("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str("100").unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 412, LastModified::None, ""));

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":3,"name":"skull3","color":"color3","icon":"icon3","unitPrice":0.3},{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2},{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}]"#,
    ));
}

#[test]
fn constraint() {
    let server = TestServer::new();

    let response = server
        .client()
        .post(
            "http://localhost/occurrence",
            r#"{
                "skull": 666,
                "amount": 1,
                "millis": 4000
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 400, LastModified::None, ""));

    let response = server
        .client()
        .post(
            "http://localhost/quick",
            r#"{
                "skull": 666,
                "amount": 1
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 400, LastModified::None, ""));
}

#[test]
fn delete_cascade() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .post(
            "http://localhost/quick",
            r#"{
                "skull": 1,
                "amount": 1
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    let quick_last_modified = check!(response_eq(
        response,
        201,
        LastModified::Gt(last_modified),
        "1"
    ))
    .unwrap();

    let response = server
        .client()
        .delete("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Gt(last_modified),
        r#"{"id":1,"name":"skull1","color":"color1","icon":"icon1","unitPrice":0.1}"#,
    ))
    .unwrap();

    let response = server
        .client()
        .get("http://localhost/quick")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        200,
        LastModified::Gt(quick_last_modified),
        r#"[]"#,
    ));
}

#[test]
fn delete_reject() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .post(
            "http://localhost/occurrence",
            r#"{
                "skull": 1,
                "amount": 1,
                "millis": 4000
            }"#,
            mime::APPLICATION_JSON,
        )
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(
        response,
        201,
        LastModified::Gt(last_modified),
        "1"
    ));

    let response = server
        .client()
        .delete("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .with_header(
            gotham::hyper::header::IF_UNMODIFIED_SINCE,
            gotham::hyper::header::HeaderValue::from_str(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
                    .as_str(),
            )
            .unwrap(),
        )
        .perform()
        .unwrap();

    check!(response_eq(response, 400, LastModified::None, ""));
}
