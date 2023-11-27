use super::{json, rmp};
use crate::{
    Error, Kind, Message, Millis, Occurrence, OccurrenceId, Payload, Push, Quick, QuickId,
    Response, ResponseWithId, Skull, SkullId,
};

#[test]
fn error_none_no_id() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Error(Error {
            kind: Kind::NotFound,
            message: None,
        }),
    });
    let json = json(&t, r#"{"error":{"kind":"NotFound"}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn error_none_with_id() {
    let t = Message::Response(ResponseWithId {
        id: Some(1),
        payload: Response::Error(Error {
            kind: Kind::NotFound,
            message: None,
        }),
    });
    let json = json(&t, r#"{"error":{"kind":"NotFound"}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn error_message() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Error(Error {
            kind: Kind::NotFound,
            message: Some(String::from("message")),
        }),
    });
    let json = json(&t, r#"{"error":{"kind":"NotFound","message":"message"}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn created() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Created),
    });
    let json = json(&t, r#""created""#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn updated() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Updated),
    });
    let json = json(&t, r#""updated""#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn deleted() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Deleted),
    });
    let json = json(&t, r#""deleted""#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skulls() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Skulls(vec![Skull {
            id: SkullId(27),
            name: String::from("name"),
            color: 1,
            icon: String::from("icon"),
            price: 1.0,
            limit: None,
        }])),
    });
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
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Skulls(Vec::new())),
    });
    let json = json(&t, r#"{"skulls":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quicks() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Quicks(vec![Quick {
            id: QuickId(27),
            skull: SkullId(72),
            amount: 1.0,
        }])),
    });
    let json = json(&t, r#"{"quicks":[{"id":27,"skull":72,"amount":1}]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quicks_empty() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Quicks(Vec::new())),
    });
    let json = json(&t, r#"{"quicks":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrences() {
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Occurrences(vec![Occurrence {
            id: OccurrenceId(27),
            skull: SkullId(72),
            amount: 1.0,
            millis: Millis(-27),
        }])),
    });
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
    let t = Message::Response(ResponseWithId {
        id: None,
        payload: Response::Payload(Payload::Occurrences(Vec::new())),
    });
    let json = json(&t, r#"{"occurrences":[]}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skull_created() {
    let t = Message::Push(Push::SkullCreated(Skull {
        id: SkullId(27),
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: None,
    }));
    let json = json(
        &t,
        r#"{"push":{"skullCreated":{"id":27,"name":"name","color":1,"icon":"icon","price":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skull_updated() {
    let t = Message::Push(Push::SkullUpdated(Skull {
        id: SkullId(27),
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: None,
    }));
    let json = json(
        &t,
        r#"{"push":{"skullUpdated":{"id":27,"name":"name","color":1,"icon":"icon","price":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skull_deleted() {
    let t = Message::Push(Push::SkullDeleted(SkullId(27)));
    let json = json(&t, r#"{"push":{"skullDeleted":27}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick_created() {
    let t = Message::Push(Push::QuickCreated(Quick {
        id: QuickId(27),
        skull: SkullId(72),
        amount: 1.0,
    }));
    let json = json(
        &t,
        r#"{"push":{"quickCreated":{"id":27,"skull":72,"amount":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick_updated() {
    let t = Message::Push(Push::QuickUpdated(Quick {
        id: QuickId(27),
        skull: SkullId(72),
        amount: 1.0,
    }));
    let json = json(
        &t,
        r#"{"push":{"quickUpdated":{"id":27,"skull":72,"amount":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick_deleted() {
    let t = Message::Push(Push::QuickDeleted(QuickId(27)));
    let json = json(&t, r#"{"push":{"quickDeleted":27}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence_created() {
    let t = Message::Push(Push::OccurrencesCreated(vec![Occurrence {
        id: OccurrenceId(27),
        skull: SkullId(72),
        amount: 1.0,
        millis: Millis(-27),
    }]));
    let json = json(
        &t,
        r#"{"push":{"occurrencesCreated":[{"id":27,"skull":72,"amount":1,"millis":-27}]}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence_created_empty() {
    let t = Message::Push(Push::OccurrencesCreated(Vec::new()));
    let json = json(&t, r#"{"push":{"occurrencesCreated":[]}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence_updated() {
    let t = Message::Push(Push::OccurrenceUpdated(Occurrence {
        id: OccurrenceId(27),
        skull: SkullId(72),
        amount: 1.0,
        millis: Millis(-27),
    }));
    let json = json(
        &t,
        r#"{"push":{"occurrenceUpdated":{"id":27,"skull":72,"amount":1,"millis":-27}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence_deleted() {
    let t = Message::Push(Push::OccurrenceDeleted(OccurrenceId(27)));
    let json = json(&t, r#"{"push":{"occurrenceDeleted":27}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
