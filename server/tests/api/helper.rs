use crate::test_utils::Assertion;

pub const USER_HEADER: &str = "X-User";
pub const EMPTY_USER: &str = "bloink-empty";

#[derive(Debug, Copy, Clone)]
pub enum LastModified {
    None,
    Eq(u64),
    Gt(u64),
    Ge(u64),
}

impl PartialEq<LastModified> for Option<u64> {
    fn eq(&self, other: &LastModified) -> bool {
        match other {
            LastModified::None => self.is_none(),
            LastModified::Eq(o) => self.map_or(false, |s| s == *o),
            LastModified::Gt(o) => self.map_or(false, |s| s > *o),
            LastModified::Ge(o) => self.map_or(false, |s| s >= *o),
        }
    }
}

#[allow(clippy::missing_panics_doc)]
pub async fn eq(
    response: hyper::Response<hyper::Body>,
    expected_status: hyper::StatusCode,
    expected_last_modified: LastModified,
    expected_body: impl AsRef<str>,
) -> Assertion<Option<u64>> {
    if response.status() != expected_status {
        return Assertion::err_ne("Status code mismatch", response.status(), expected_status);
    }

    let last_modified = extract_last_modified(&response);
    if last_modified != expected_last_modified {
        return Assertion::err_ne(
            "Last modified mismatch",
            last_modified,
            expected_last_modified,
        );
    }

    let expected_body = expected_body.as_ref();
    let body = extract_body(response).await;
    if body != expected_body {
        return Assertion::err_ne("Body mismatch", body, expected_body);
    }

    Assertion::Ok(last_modified)
}

pub async fn extract_body(response: hyper::Response<hyper::Body>) -> String {
    String::from_utf8(
        hyper::body::to_bytes(response.into_body())
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

#[allow(clippy::missing_panics_doc)]
pub fn extract_last_modified(response: &hyper::Response<hyper::Body>) -> Option<u64> {
    response
        .headers()
        .get(hyper::header::LAST_MODIFIED)
        .map(|h| h.to_str().unwrap().parse().unwrap())
}

pub fn build_skull_payload<const N: usize>(ids: [u8; N]) -> String {
    let items = ids
        .map(|j| {
            format!(
                r#"{{"id":{j},"name":"skull{j}","color":"color{j}","icon":"icon{j}","unitPrice":0.{j}}}{}"#,
                if j < 3 { "," } else { "" }
            )
        })
        .into_iter()
        .collect::<String>();

    format!("[{items}]")
}
