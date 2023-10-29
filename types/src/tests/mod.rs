use crate::{Millis, Occurrence, OccurrenceId, Quick, QuickId, Skull, SkullId};

mod error;
mod request;
mod response;
mod ws;

fn json<T: PartialEq + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned>(
    t: &T,
    expected: &str,
) -> Result<T, String> {
    let string = serde_json::to_string(t).map_err(|e| e.to_string())?;
    let back = serde_json::from_str::<T>(expected).map_err(|e| e.to_string())?;
    if t != &back {
        eprintln!("Got:    {back:#?}");
        eprintln!("Wanted: {t:#?}");
        return Err(String::from("Mismatch"));
    }
    serde_json::from_str(&string).map_err(|e| e.to_string())
}

fn rmp<T: serde::Serialize + serde::de::DeserializeOwned>(
    t: &T,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = rmp_serde::to_vec(t)?;
    let back = rmp_serde::from_slice(&bytes)?;
    Ok(back)
}

#[test]
fn skull_id() {
    let t = SkullId(27);
    let json = json(&t, "27").unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skull_none() {
    let t = Skull {
        id: SkullId(27),
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: None,
    };
    let json = json(
        &t,
        r#"{"id":27,"name":"name","color":1,"icon":"icon","price":1}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn skull_limit() {
    let t = Skull {
        id: SkullId(27),
        name: String::from("name"),
        color: 1,
        icon: String::from("icon"),
        price: 1.0,
        limit: Some(1.0),
    };
    let json = json(
        &t,
        r#"{"id":27,"name":"name","color":1,"icon":"icon","price":1,"limit":1}"#,
    )
    .unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick_id() {
    let t = QuickId(27);
    let json = json(&t, "27").unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn quick() {
    let t = Quick {
        id: QuickId(27),
        skull: SkullId(72),
        amount: 2.7,
    };
    let json = json(&t, r#"{"id":27,"skull":72,"amount":2.7}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence_id() {
    let t = OccurrenceId(27);
    let json = json(&t, "27").unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn occurrence() {
    let t = Occurrence {
        id: OccurrenceId(27),
        skull: SkullId(72),
        amount: 2.7,
        millis: Millis(-27),
    };
    let json = json(&t, r#"{"id":27,"skull":72,"amount":2.7,"millis":-27}"#).unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}

#[test]
fn millis() {
    let t = Millis(1);
    let json = json(&t, "1").unwrap();
    let rmp = rmp(&t).unwrap();

    assert_eq!(t, json);
    assert_eq!(t, rmp);
}
