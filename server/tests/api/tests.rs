use crate::check_async as check;
use hyper::StatusCode;

use crate::{
    client::Client,
    helper::{
        build_skull_payload, eq, extract_last_modified, LastModified, EMPTY_USER, USER_HEADER,
    },
    server,
};

pub fn test<'a>(
    runtime: &'a tokio::runtime::Runtime,
    server: &'a server::Server,
) -> Vec<(&'static str, Result<(), tokio::task::JoinError>)> {
    macro_rules! test {
        ($test: path) => {
            (
                concat!("api::", stringify!($test)),
                runtime.spawn($test(server.client())).await,
            )
        };
    }

    runtime.block_on(async {
        vec![
            test!(missing_user),
            test!(unknown_user),
            test!(method_not_allowed),
            test!(not_found),
            test!(head),
            test!(list),
            test!(list_empty),
            test!(list_limited),
            test!(list_bad_request),
            test!(create),
            test!(create_constraint),
            test!(create_conflict),
            test!(create_bad_payload),
            test!(create_length_required),
            test!(create_too_large),
            test!(read),
            test!(read_not_found),
            test!(update),
            test!(update_not_found),
            test!(update_constraint),
            test!(update_conflict),
            test!(update_out_of_sync),
            test!(update_unmodified_missing),
            test!(update_bad_payload),
            test!(update_length_required),
            test!(update_too_large),
            test!(delete),
            test!(delete_not_found),
            test!(delete_rejected),
            test!(delete_out_of_sync),
            test!(delete_unmodified_missing),
            test!(json::skull),
            test!(json::quick),
            test!(json::occurrence),
            test!(json::list),
        ]
    })
}

async fn missing_user(client: Client) {
    let response = client
        .get_with("/skull", |r| {
            r.headers_mut().remove(USER_HEADER);
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, LastModified::None, ""));
}

async fn unknown_user(client: Client) {
    let response = client
        .get_with("/skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, "unknown".try_into().unwrap());
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, LastModified::None, ""));
}

async fn method_not_allowed(client: Client) {
    let response = client
        .get_with("/skull", |r| *r.method_mut() = hyper::Method::PATCH)
        .await;

    check!(eq(
        response,
        StatusCode::METHOD_NOT_ALLOWED,
        LastModified::None,
        ""
    ));
}

async fn not_found(client: Client) {
    let response = client.get("/bloink").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

async fn head(client: Client) {
    let response = client.head("/skull").await;

    check!(eq(response, StatusCode::OK, LastModified::Gt(0), ""));
}

async fn list(client: Client) {
    let last_modified = client.last_modified("/skull").await;
    let response = client.get("/skull").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Eq(last_modified),
        build_skull_payload([1, 2, 3])
    ));
}

async fn list_empty(client: Client) {
    let response = client
        .head_with("/skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, EMPTY_USER.try_into().unwrap());
        })
        .await;
    let last_modified = extract_last_modified(&response).unwrap();
    let response = client
        .get_with("/skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, EMPTY_USER.try_into().unwrap());
        })
        .await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Eq(last_modified),
        "[]"
    ));
}

async fn list_limited(client: Client) {
    let last_modified = client.last_modified("/skull").await;

    for i in 0..5 {
        let response = client.get(format!("/skull?limit={i}")).await;

        let payload = match i {
            0 => build_skull_payload([]),
            1 => build_skull_payload([3]),
            2 => build_skull_payload([2, 3]),
            _ => build_skull_payload([1, 2, 3]),
        };

        check!(eq(
            response,
            StatusCode::OK,
            LastModified::Eq(last_modified),
            payload
        ));
    }
}

async fn list_bad_request(client: Client) {
    let response = client.get("/skull?limit=").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn create(client: Client) {
    let last_modified = client.last_modified("/quick").await;

    let response = client
        .post(
            "/quick",
            r#"{
                "skull": 1,
                "amount": 27
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::CREATED,
        LastModified::Gt(last_modified),
        "4"
    ));
}

async fn create_constraint(client: Client) {
    let response = client
        .post(
            "/quick",
            r#"{
                "skull": 27,
                "amount": 27
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn create_conflict(client: Client) {
    let response = client
        .post(
            "/quick",
            r#"{
                "skull": 1,
                "amount": 1,
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn create_bad_payload(client: Client) {
    let response = client.post("/quick", r#"{"bloink": 27}"#).await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn create_length_required(client: Client) {
    let response = client.post("/quick", hyper::Body::empty()).await;

    check!(eq(
        response,
        StatusCode::LENGTH_REQUIRED,
        LastModified::None,
        ""
    ));
}
async fn create_too_large(client: Client) {
    let response = client.post("/occurrence", [0_u8; 1025].as_slice()).await;

    check!(eq(
        response,
        StatusCode::PAYLOAD_TOO_LARGE,
        LastModified::None,
        ""
    ));
}

async fn read(client: Client) {
    let last_modified = client.last_modified("/skull").await;
    let response = client.get("/skull/2").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Eq(last_modified),
        r#"{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2}"#
    ));
}

async fn read_not_found(client: Client) {
    let response = client.get("/skull/27").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

async fn update(client: Client) {
    let last_modified = client.last_modified("/quick").await;
    let response = client
        .put(
            "/quick/3",
            r#"{
                "skull": 3,
                "amount": 27
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Gt(last_modified),
        r#"{"id":3,"skull":3,"amount":3.0}"#,
    ));
}

async fn update_not_found(client: Client) {
    let response = client
        .put(
            "/quick/27",
            r#"{
                "skull": 3,
                "amount": 27
            }"#,
        )
        .await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

async fn update_constraint(client: Client) {
    let response = client
        .put(
            "/quick/1",
            r#"{
                "skull": 27,
                "amount": 1
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn update_conflict(client: Client) {
    let response = client
        .put(
            "/quick/1",
            r#"{
                "skull": 2,
                "amount": 2
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn update_out_of_sync(client: Client) {
    let response = client
        .put_with(
            "/quick/3",
            r#"{
                "skull": 3,
                "amount": 27
            }"#,
            |r| {
                r.headers_mut()
                    .insert(hyper::header::IF_UNMODIFIED_SINCE, 1.try_into().unwrap());
            },
        )
        .await;

    check!(eq(
        response,
        StatusCode::PRECONDITION_FAILED,
        LastModified::None,
        ""
    ));
}

async fn update_unmodified_missing(client: Client) {
    let response = client
        .put_with(
            "/quick/3",
            r#"{
                "skull": 3,
                "amount": 27
            }"#,
            |r| {
                r.headers_mut().remove(hyper::header::IF_UNMODIFIED_SINCE);
            },
        )
        .await;

    check!(eq(
        response,
        StatusCode::PRECONDITION_FAILED,
        LastModified::None,
        ""
    ));
}

async fn update_bad_payload(client: Client) {
    let response = client
        .put(
            "/quick/1",
            r#"{
                "amount": 2
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn update_length_required(client: Client) {
    let response = client.put("/quick/1", hyper::Body::empty()).await;

    check!(eq(
        response,
        StatusCode::LENGTH_REQUIRED,
        LastModified::None,
        ""
    ));
}

async fn update_too_large(client: Client) {
    let response = client.put("/quick/1", [0_u8; 1025].as_slice()).await;

    check!(eq(
        response,
        StatusCode::PAYLOAD_TOO_LARGE,
        LastModified::None,
        ""
    ));
}

async fn delete(client: Client) {
    let last_modified = client.last_modified("/occurrence").await;
    let response = client.delete("/occurrence/3").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Gt(last_modified),
        r#"{"id":3,"skull":3,"amount":3.0,"millis":3}"#
    ));
}

async fn delete_not_found(client: Client) {
    let response = client.delete("/occurrence/27").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

async fn delete_rejected(client: Client) {
    let response = client.delete("/skull/1").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

async fn delete_out_of_sync(client: Client) {
    let response = client
        .delete_with("/occurrence/3", |r| {
            r.headers_mut()
                .insert(hyper::header::IF_UNMODIFIED_SINCE, 1.try_into().unwrap());
        })
        .await;

    check!(eq(
        response,
        StatusCode::PRECONDITION_FAILED,
        LastModified::None,
        ""
    ));
}

async fn delete_unmodified_missing(client: Client) {
    let response = client
        .delete_with("/occurrence/3", |r| {
            r.headers_mut().remove(hyper::header::IF_UNMODIFIED_SINCE);
        })
        .await;

    check!(eq(
        response,
        StatusCode::PRECONDITION_FAILED,
        LastModified::None,
        ""
    ));
}

mod json {
    use serde_json::{Number, Value};

    use crate::{client::Client, helper::extract_body};

    pub async fn skull(client: Client) {
        let response = client.get("/skull/1").await;
        let body = extract_body(response).await;
        let data =
            serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&body)
                .unwrap();

        assert_eq!(data.keys().len(), 5);
        assert_eq!(data["id"], Value::Number(Number::from(1)));
        assert_eq!(data["name"], Value::String(String::from("skull1")));
        assert_eq!(data["color"], Value::String(String::from("color1")));
        assert_eq!(data["icon"], Value::String(String::from("icon1")));
        assert_eq!(
            data["unitPrice"],
            Value::Number(Number::from_f64(0.1).unwrap())
        );
    }

    pub async fn quick(client: Client) {
        let response = client.get("/quick/1").await;
        let body = extract_body(response).await;
        let data =
            serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&body)
                .unwrap();

        assert_eq!(data.keys().len(), 3);
        assert_eq!(data["id"], Value::Number(Number::from(1)));
        assert_eq!(data["skull"], Value::Number(Number::from(1)));
        assert_eq!(
            data["amount"],
            Value::Number(Number::from_f64(1.0).unwrap())
        );
    }

    pub async fn occurrence(client: Client) {
        let response = client.get("/occurrence/1").await;
        let body = extract_body(response).await;
        let data =
            serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&body)
                .unwrap();

        assert_eq!(data.keys().len(), 4);
        assert_eq!(data["id"], Value::Number(Number::from(1)));
        assert_eq!(data["skull"], Value::Number(Number::from(1)));
        assert_eq!(
            data["amount"],
            Value::Number(Number::from_f64(1.0).unwrap())
        );
        assert_eq!(data["millis"], Value::Number(Number::from(1)));
    }

    pub async fn list(client: Client) {
        let response = client.get("/skull").await;
        let body = extract_body(response).await;
        let data =
            serde_json::from_str::<Vec<std::collections::HashMap<String, serde_json::Value>>>(
                &body,
            )
            .unwrap();

        assert_eq!(data.len(), 3);
        for (i, d) in data.iter().enumerate() {
            let i = i + 1;
            assert_eq!(d["name"], Value::String(format!("skull{i}")));
            assert_eq!(d["color"], Value::String(format!("color{i}")));
            assert_eq!(d["icon"], Value::String(format!("icon{i}")));
            assert_eq!(
                d["unitPrice"],
                Value::Number(Number::from_f64(format!("0.{i}").parse().unwrap()).unwrap())
            );
        }
    }
}
