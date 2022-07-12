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

struct TestServer(gotham::test::TestServer, std::path::PathBuf);

impl TestServer {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(rand::random::<u64>().to_string());
        std::fs::create_dir(&dir).unwrap();
        let transfer_dir = CopiablePath::new(&dir);

        let server = gotham::test::TestServer::new(move || {
            super::route(crate::options::Options {
                port: 0,
                threads: 0,
                cors: None,
                store_path: Some(transfer_dir.into_path()),
                web_path: None,
                users: vec![String::from(USER)],
            })
        })
        .unwrap();

        Self(server, dir)
    }

    fn populate(self) -> Self {
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

        assert_eq!(
            serde_json::from_str::<Vec<crate::store::Skull>>(
                self.client()
                    .get("http://localhost/skull")
                    .with_header(
                        super::mapper::request::USER_HEADER,
                        gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
                    )
                    .perform()
                    .unwrap()
                    .read_utf8_body()
                    .unwrap()
                    .as_str(),
            )
            .unwrap()
            .len(),
            3
        );

        self
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.1));
    }
}

impl std::ops::Deref for TestServer {
    type Target = gotham::test::TestServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn forbidden() {
    let server = TestServer::new();

    assert_eq!(
        server
            .client()
            .get("http://localhost/skull")
            .with_header(
                super::mapper::request::USER_HEADER,
                gotham::hyper::header::HeaderValue::from_str("").unwrap(),
            )
            .perform()
            .unwrap()
            .status(),
        403
    );
}

#[test]
fn unrecognized_skulls_are_allowed() {
    let server = TestServer::new().populate();

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
            super::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();
    assert_eq!(response.status(), 201);
    let body = response.read_utf8_body().unwrap();
    assert_eq!(body, String::from("0"));
}

#[test]
fn list() {
    let server = TestServer::new().populate();

    let response = server
        .client()
        .get("http://localhost/skull")
        .with_header(
            super::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = response.read_utf8_body().unwrap();
    assert_eq!(
        body,
        String::from(
            r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1},{"id":1,"name":"skull2","color":"","icon":"","unitPrice":0.2},{"id":2,"name":"skull3","color":"","icon":"","unitPrice":0.3}]"#
        )
    );
}

#[test]
fn list_limited() {
    let server = TestServer::new().populate();

    let response = server
        .client()
        .get("http://localhost/skull?limit=1")
        .with_header(
            super::mapper::request::USER_HEADER,
            gotham::hyper::header::HeaderValue::from_str(USER).unwrap(),
        )
        .perform()
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = response.read_utf8_body().unwrap();
    assert_eq!(
        body,
        String::from(r#"[{"id":0,"name":"skull1","color":"","icon":"","unitPrice":0.1}]"#)
    );
}
