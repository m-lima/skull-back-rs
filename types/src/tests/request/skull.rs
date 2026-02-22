use crate::{
    Request, Setter, SkullId,
    request::{
        Skull,
        skull::{Create, Delete, Update},
    },
    tests::{json, rmp},
};

#[test]
fn list() {
    let t = Request::Skull(Skull::List);
    let json = json(&t, r#"{"skull":"list"}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn create_none() {
    let t = Request::Skull(Skull::Create(Create {
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: None,
    }));
    let json = json(
        &t,
        r#"{"skull":{"create":{"name":"name","color":1,"icon":"icon","price":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn create_limit() {
    let t = Request::Skull(Skull::Create(Create {
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: Some(1.0),
    }));
    let json = json(
        &t,
        r#"{"skull":{"create":{"name":"name","color":1,"icon":"icon","price":1,"limit":1}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_none() {
    let t = Request::Skull(Skull::Update(Update {
        id: SkullId(27),
        name: Some(Setter {
            set: String::from("name"),
        }),
        color: None,
        icon: None,
        price: Some(Setter { set: 1.0 }),
        limit: None,
    }));
    let json = json(
        &t,
        r#"{"skull":{"update":{"id":27,"name":{"set":"name"},"price":{"set":1}}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_set_none() {
    let t = Request::Skull(Skull::Update(Update {
        id: SkullId(27),
        name: Some(Setter {
            set: String::from("name"),
        }),
        color: None,
        icon: None,
        price: Some(Setter { set: 1.0 }),
        limit: Some(Setter { set: None }),
    }));
    let json = json(
        &t,
        r#"{"skull":{"update":{"id":27,"name":{"set":"name"},"price":{"set":1},"limit":{}}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn update_set_limit() {
    let t = Request::Skull(Skull::Update(Update {
        id: SkullId(27),
        name: Some(Setter {
            set: String::from("name"),
        }),
        color: None,
        icon: None,
        price: Some(Setter { set: 1.0 }),
        limit: Some(Setter { set: Some(1.0) }),
    }));
    let json = json(
        &t,
        r#"{"skull":{"update":{"id":27,"name":{"set":"name"},"price":{"set":1},"limit":{"set":1.0}}}}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn delete() {
    let t = Request::Skull(Skull::Delete(Delete { id: SkullId(27) }));
    let json = json(&t, r#"{"skull":{"delete":{"id":27}}}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
