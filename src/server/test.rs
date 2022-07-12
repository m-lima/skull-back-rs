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

struct TestServer(gotham::test::TestServer, Option<std::path::PathBuf>);

impl TestServer {
    fn new(dir: impl Into<Option<std::path::PathBuf>>) -> Self {
        let dir = dir
            .into()
            .map(|p| p.join(rand::random::<u64>().to_string()));
        let transfer_dir = dir.as_ref().map(|dir| {
            std::fs::create_dir(dir).unwrap();
            CopiablePath::new(dir)
        });

        let server = gotham::test::TestServer::new(move || {
            super::route(crate::options::Options {
                port: 0,
                threads: 0,
                cors: None,
                store_path: transfer_dir.map(CopiablePath::into_path),
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
        if let Some(dir) = &self.1 {
            drop(std::fs::remove_dir_all(dir));
        }
    }
}

impl std::ops::Deref for TestServer {
    type Target = gotham::test::TestServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod tests {
    use super::{TestServer, USER};

    pub fn forbidden(dir: impl Into<Option<std::path::PathBuf>>) {
        let server = TestServer::new(dir);

        let response = server
            .client()
            .get("http://localhost/skull")
            .with_header(
                crate::server::mapper::request::USER_HEADER,
                gotham::hyper::header::HeaderValue::from_str("").unwrap(),
            )
            .perform()
            .unwrap();
        assert_eq!(response.status(), 403);
        let body = response.read_utf8_body().unwrap();
        assert_eq!(body, String::new());
    }

    pub fn unrecognized_skulls_are_allowed(dir: impl Into<Option<std::path::PathBuf>>) {
        let server = TestServer::new(dir);

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
        assert_eq!(response.status(), 201);
        let body = response.read_utf8_body().unwrap();
        assert_eq!(body, String::from("0"));
    }

    pub fn bad_request(dir: impl Into<Option<std::path::PathBuf>>) {
        let server = TestServer::new(dir);

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
        assert_eq!(response.status(), 400);
        let body = response.read_utf8_body().unwrap();
        assert_eq!(body, String::new());
    }

    pub fn list(dir: impl Into<Option<std::path::PathBuf>>) {
        let server = TestServer::new(dir).populate();

        let response = server
            .client()
            .get("http://localhost/skull")
            .with_header(
                crate::server::mapper::request::USER_HEADER,
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

    pub fn list_limited(dir: impl Into<Option<std::path::PathBuf>>) {
        let server = TestServer::new(dir).populate();

        let response = server
            .client()
            .get("http://localhost/skull?limit=1")
            .with_header(
                crate::server::mapper::request::USER_HEADER,
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
}

macro_rules! impl_test {
    ($dir:expr, $mod:ident) => {
        mod $mod {
            use super::tests;

            #[test]
            fn forbidden() {
                tests::forbidden($dir);
            }

            #[test]
            fn unrecognized_skulls_are_allowed() {
                tests::unrecognized_skulls_are_allowed($dir);
            }

            #[test]
            fn bad_request() {
                tests::bad_request($dir);
            }

            #[test]
            fn list() {
                tests::list($dir);
            }

            #[test]
            fn list_limited() {
                tests::list_limited($dir);
            }
        }
    };
}

impl_test!(None, in_memory);
impl_test!(std::env::temp_dir(), in_file);
