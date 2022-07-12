const USER: &str = "bloink";

struct TestServer(gotham::test::TestServer);

impl TestServer {
    fn new() -> Self {
        // let dir = std::env::temp_dir();
        let server = gotham::test::TestServer::new(|| {
            super::route(crate::options::Options {
                port: 0,
                threads: 0,
                cors: None,
                store_path: None,
                web_path: None,
                users: vec![String::from(USER)],
            })
        })
        .unwrap();

        Self(server)
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

        self
    }
}

// impl Drop for TestServer {
//     fn drop(&mut self) {
//         drop(std::fs::remove_dir_all(&self.1));
//     }
// }

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
    assert_eq!(body, String::from("3"));
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
    assert_eq!(body, String::from("[]"));
}
