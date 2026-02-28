use crate::{
    Millis, OccurrenceId, Request, Setter, SkullId,
    request::{
        Occurrence,
        occurrence::{Create, Delete, Item, Search, Update},
    },
    tests::{json, rmp},
};

#[test]
fn list() {
    let t = Request::Occurrence(Occurrence::List);
    let json = json(&t, r#"{"occurrence":"list"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick() {
    let t = Request::Occurrence(Occurrence::Quick);
    let json = json(&t, r#"{"occurrence":"quick"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn search_none() {
    let t = Request::Occurrence(Occurrence::Search(Search {
        skulls: None,
        start: None,
        end: None,
        limit: None,
    }));
    let json = json(&t, r#"{"occurrence":{"search":{}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn create_empty() {
    let t = Request::Occurrence(Occurrence::Create(Create { items: Vec::new() }));
    let json = json(&t, r#"{"occurrence":{"create":{"items":[]}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn create() {
    let t = Request::Occurrence(Occurrence::Create(Create {
        items: vec![Item {
            skull: SkullId(27),
            amount: 1.0,
            millis: Millis(72),
        }],
    }));
    let json = json(
        &t,
        r#"{"occurrence":{"create":{"items":[{"skull":27,"amount":1,"millis":72}]}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_none() {
    let t = Request::Occurrence(Occurrence::Update(Update {
        id: OccurrenceId(27),
        skull: None,
        amount: None,
        millis: None,
    }));
    let json = json(&t, r#"{"occurrence":{"update":{"id":27}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_some() {
    let t = Request::Occurrence(Occurrence::Update(Update {
        id: OccurrenceId(27),
        skull: Some(Setter { set: SkullId(72) }),
        amount: Some(Setter { set: 1.0 }),
        millis: Some(Setter { set: Millis(-27) }),
    }));
    let json = json(
        &t,
        r#"{"occurrence":{"update":{"id":27,"skull":{"set":72},"amount":{"set":1},"millis":{"set":-27}}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn delete() {
    let t = Request::Occurrence(Occurrence::Delete(Delete {
        id: OccurrenceId(27),
    }));
    let json = json(&t, r#"{"occurrence":{"delete":{"id":27}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
