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
        let path = std::env::temp_dir().join(rand::random::<u64>().to_string());
        std::fs::create_dir(&path).unwrap();
        let copiable_path = CopiablePath::new(&path);

        let server = gotham::test::TestServer::new(move || {
            super::route(crate::options::Options {
                port: 0,
                threads: 0,
                cors: None,
                store_path: Some(copiable_path.into_path()),
                web_path: None,
                users: vec![String::from(USER)],
            })
        })
        .unwrap();

        Self { server, path }
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
                        "color": "",
                        "icon": "",
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
                        "color": "",
                        "icon": "",
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
                        "color": "",
                        "icon": "",
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

fn assert_response(
    response: gotham::test::TestResponse,
    expected_status: u16,
    expected_last_modified: LastModified,
    expected_body: &str,
) -> Option<u64> {
    assert_eq!(response.status(), expected_status);

    let last_modified = response
        .headers()
        .get(gotham::hyper::header::LAST_MODIFIED)
        .map(|h| h.to_str().unwrap().parse().unwrap());
    assert_eq!(last_modified, expected_last_modified);

    let body = response.read_utf8_body().unwrap();
    assert_eq!(body, expected_body);

    last_modified
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

    assert_response(response, 403, LastModified::None, "");
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

    assert_response(response, 200, LastModified::Eq(last_modified), "[]");
}

#[test]
fn unrecognized_skulls_are_allowed() {
    let server = TestServer::new();
    let last_modified = server.last_modified();

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

    assert_response(response, 201, LastModified::Gt(last_modified), "0");
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

    assert_response(response, 400, LastModified::None, "");
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

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
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

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1}]"#,
    );
}

#[test]
fn read() {
    let server = TestServer::new();
    let last_modified = server.populate();

    let response = server
        .client()
        .get("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2}"#,
    );
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

    assert_response(response, 404, LastModified::None, "");
}

#[test]
fn update() {
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

    let last_modified = assert_response(
        response,
        200,
        LastModified::Gt(last_modified),
        r#"{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2}"#,
    )
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

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull4","color":"","icon":"","unitPrice":0.4},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
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

    assert_response(response, 404, LastModified::None, "");

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
}

#[test]
fn delete() {
    let server = TestServer::new();
    let last_modified = server.populate();

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

    let last_modified = assert_response(
        response,
        200,
        LastModified::Gt(last_modified),
        r#"{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2}"#,
    )
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

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
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

    assert_response(response, 404, LastModified::None, "");

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
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

    assert_response(response, 412, LastModified::None, "");

    let response = server
        .client()
        .delete("http://localhost/skull/1")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(response, 412, LastModified::None, "");

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
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

    assert_response(response, 412, LastModified::None, "");

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

    assert_response(response, 412, LastModified::None, "");

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            crate::server::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();

    assert_response(
        response,
        200,
        LastModified::Eq(last_modified),
        r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#,
    );
}
