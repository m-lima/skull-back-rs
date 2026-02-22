use crate::{
    QuickId, Request, Setter, SkullId,
    request::{
        Quick,
        quick::{Create, Delete, Update},
    },
    tests::{json, rmp},
};

#[test]
fn list() {
    let t = Request::Quick(Quick::List);
    let json = json(&t, r#"{"quick":"list"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn create() {
    let t = Request::Quick(Quick::Create(Create {
        skull: SkullId(27),
        amount: 1.0,
    }));
    let json = json(&t, r#"{"quick":{"create":{"skull":27,"amount":1}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_none() {
    let t = Request::Quick(Quick::Update(Update {
        id: QuickId(27),
        skull: None,
        amount: None,
    }));
    let json = json(&t, r#"{"quick":{"update":{"id":27}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_some() {
    let t = Request::Quick(Quick::Update(Update {
        id: QuickId(27),
        skull: Some(Setter { set: SkullId(72) }),
        amount: Some(Setter { set: 1.0 }),
    }));
    let json = json(
        &t,
        r#"{"quick":{"update":{"id":27,"skull":{"set":72},"amount":{"set":1}}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn delete() {
    let t = Request::Quick(Quick::Delete(Delete { id: QuickId(27) }));
    let json = json(&t, r#"{"quick":{"delete":{"id":27}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
