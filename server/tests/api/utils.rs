use crate::test_utils::Assertion;

pub const USER_HEADER: &str = "X-User";
pub const USER: &str = "bloink";
pub const EMPTY_USER: &str = "bloink-empty";

pub struct TestPath(std::path::PathBuf);

impl TestPath {
    #[must_use]
    pub fn new() -> Self {
        let name = format!(
            "{:016x}{:016x}",
            rand::random::<u64>(),
            rand::random::<u64>()
        );
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
        Self(path)
    }
}

impl Default for TestPath {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for TestPath {
    type Target = std::path::PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TestPath {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

#[allow(clippy::missing_panics_doc)]
pub async fn eq(
    response: reqwest::Response,
    expected_status: hyper::StatusCode,
    expected_body: impl AsRef<str>,
) -> Assertion {
    if response.status() != expected_status {
        return Assertion::err_ne("Status code mismatch", response.status(), expected_status);
    }

    let expected_body = expected_body.as_ref();
    let body = extract_body(response).await;
    if body != expected_body {
        return Assertion::err_ne("Body mismatch", body, expected_body);
    }

    Assertion::Ok
}

pub async fn extract_body(response: reqwest::Response) -> String {
    response.text().await.unwrap()
}

pub fn build_skull_payload<const N: usize>(ids: [u8; N]) -> String {
    let items = ids
        .map(|j| format!(r#"{{"id":{j},"name":"skull{j}","color":{j},"icon":"icon{j}","price":0.{j},"limit":null}}"#))
        .into_iter()
        .collect::<Vec<_>>().join(",");

    format!("{{\"skulls\":[{items}]}}")
}

pub fn build_occurrence_payload<const N: usize>(ids: [u8; N]) -> String {
    let items = ids
        .map(|j| format!(r#"{{"id":{j},"skull":{j},"amount":{j}.0,"millis":{j}}}"#))
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");

    format!("{{\"occurrences\":[{items}]}}")
}
