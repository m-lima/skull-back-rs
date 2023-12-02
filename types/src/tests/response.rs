use crate::{
    Change, Error, Kind, Millis, Occurrence, OccurrenceId, Payload, Quick, QuickId, Response,
    Skull, SkullId,
};

use super::{json, rmp};

#[test]
fn error_none() {
    let t = Response::Error(Error {
        kind: Kind::NotFound,
        message: None,
    });
    let json = json(&t, r#"{"error":{"kind":"NotFound"}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn error_message() {
    let t = Response::Error(Error {
        kind: Kind::NotFound,
        message: Some(String::from("message")),
    });
    let json = json(&t, r#"{"error":{"kind":"NotFound","message":"message"}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn created() {
    let t = Response::Payload(Payload::Change(Change::Created));
    let json = json(&t, r#"{"change":"created"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn updated() {
    let t = Response::Payload(Payload::Change(Change::Updated));
    let json = json(&t, r#"{"change":"updated"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn deleted() {
    let t = Response::Payload(Payload::Change(Change::Deleted));
    let json = json(&t, r#"{"change":"deleted"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skulls() {
    let t = Response::Payload(Payload::Skulls(vec![Skull {
        id: SkullId(27),
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: None,
    }]));
    let json = json(
        &t,
        r#"{"skulls":[{"id":27,"name":"name","color":1,"icon":"icon","price":1}]}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skulls_empty() {
    let t = Response::Payload(Payload::Skulls(Vec::new()));
    let json = json(&t, r#"{"skulls":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quicks() {
    let t = Response::Payload(Payload::Quicks(vec![Quick {
        id: QuickId(27),
        skull: SkullId(72),
        amount: 1.0,
    }]));
    let json = json(&t, r#"{"quicks":[{"id":27,"skull":72,"amount":1}]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quicks_empty() {
    let t = Response::Payload(Payload::Quicks(Vec::new()));
    let json = json(&t, r#"{"quicks":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrences() {
    let t = Response::Payload(Payload::Occurrences(vec![Occurrence {
        id: OccurrenceId(27),
        skull: SkullId(72),
        amount: 1.0,
        millis: Millis(-27),
    }]));
    let json = json(
        &t,
        r#"{"occurrences":[{"id":27,"skull":72,"amount":1,"millis":-27}]}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrences_empty() {
    let t = Response::Payload(Payload::Occurrences(Vec::new()));
    let json = json(&t, r#"{"occurrences":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
