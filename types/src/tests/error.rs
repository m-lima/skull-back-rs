use crate::{Error, Kind};

use super::{json, rmp};

#[test]
fn error_none() {
    let t = Error {
        kind: Kind::NotFound,
        message: None,
    };
    let json = json(&t, r#"{"kind":"NotFound"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn error_message() {
    let t = Error {
        kind: Kind::NotFound,
        message: Some(String::from("message")),
    };
    let json = json(&t, r#"{"kind":"NotFound","message":"message"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
