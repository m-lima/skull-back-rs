mod helper;

use helper::{
    build_skull_payload, eq, extract_body, extract_last_modified, server::Server, LastModified,
    EMPTY_USER, USER_HEADER,
};
use hyper::StatusCode;
use test_utils::check_async as check;

#[tokio::test(flavor = "multi_thread")]
async fn missing_user() {
    let server = Server::instance().await;

    let response = server
        .get_with("/skull", |r| {
            r.headers_mut().remove(USER_HEADER);
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, LastModified::None, ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_user() {
    let server = Server::instance().await;

    let response = server
        .get_with("/skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, "unknown".try_into().unwrap());
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, LastModified::None, ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn method_not_allowed() {
    let server = Server::instance().await;

    let response = server
        .get_with("/skull", |r| *r.method_mut() = hyper::Method::PATCH)
        .await;

    check!(eq(
        response,
        StatusCode::METHOD_NOT_ALLOWED,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn not_found() {
    let server = Server::instance().await;

    let response = server.get("/bloink").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn head() {
    let server = Server::instance().await;

    let response = server.head("/skull").await;

    check!(eq(response, StatusCode::OK, LastModified::Gt(0), ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn list() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/skull").await;
    let response = server.get("/skull").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Eq(last_modified),
        build_skull_payload([1, 2, 3])
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_empty() {
    let server = Server::instance().await;

    let response = server
        .head_with("/skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, EMPTY_USER.try_into().unwrap());
        })
        .await;
    let last_modified = extract_last_modified(&response).unwrap();
    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn list_limited() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/skull").await;

    for i in 0..5 {
        let response = server.get(format!("/skull?limit={i}")).await;

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

#[tokio::test(flavor = "multi_thread")]
async fn list_bad_request() {
    let server = Server::instance().await;

    let response = server.get("/skull?limit=").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/quick").await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn create_constraint() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn create_conflict() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn create_bad_payload() {
    let server = Server::instance().await;

    let response = server.post("/skull", r#"{"bloink": 27}"#).await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_length_required() {
    let server = Server::instance().await;

    let response = server.post("/skull", hyper::Body::empty()).await;

    check!(eq(
        response,
        StatusCode::LENGTH_REQUIRED,
        LastModified::None,
        ""
    ));
}
#[tokio::test(flavor = "multi_thread")]
async fn create_too_large() {
    let server = Server::instance().await;

    let response = server.post("/occurrence", [0_u8; 1025].as_slice()).await;

    check!(eq(
        response,
        StatusCode::PAYLOAD_TOO_LARGE,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn read() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/skull").await;
    let response = server.get("/skull/2").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Eq(last_modified),
        r#"{"id":2,"name":"skull2","color":"color2","icon":"icon2","unitPrice":0.2}"#
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn read_not_found() {
    let server = Server::instance().await;

    let response = server.get("/skull/27").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn update() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/quick").await;
    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_not_found() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_constraint() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_conflict() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_out_of_sync() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_unmodified_missing() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_bad_payload() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn update_length_required() {
    let server = Server::instance().await;

    let response = server.put("/quick/1", hyper::Body::empty()).await;

    check!(eq(
        response,
        StatusCode::LENGTH_REQUIRED,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_too_large() {
    let server = Server::instance().await;

    let response = server.put("/quick/1", [0_u8; 1025].as_slice()).await;

    check!(eq(
        response,
        StatusCode::PAYLOAD_TOO_LARGE,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete() {
    let server = Server::instance().await;

    let last_modified = server.last_modified("/occurrence").await;
    let response = server.delete("/occurrence/3").await;

    check!(eq(
        response,
        StatusCode::OK,
        LastModified::Gt(last_modified),
        r#"{"id":3,"skull":3,"amount":3.0,"millis":3}"#
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_not_found() {
    let server = Server::instance().await;

    let response = server.delete("/occurrence/27").await;

    check!(eq(response, StatusCode::NOT_FOUND, LastModified::None, ""));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_rejected() {
    let server = Server::instance().await;

    let response = server.delete("/skull/1").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        LastModified::None,
        ""
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_out_of_sync() {
    let server = Server::instance().await;

    let response = server
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

#[tokio::test(flavor = "multi_thread")]
async fn delete_unmodified_missing() {
    let server = Server::instance().await;

    let response = server
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
    use super::{extract_body, Server};
    use serde_json::{Number, Value};

    #[tokio::test(flavor = "multi_thread")]
    async fn skull() {
        let server = Server::instance().await;

        let response = server.get("/skull/1").await;
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

    #[tokio::test(flavor = "multi_thread")]
    async fn quick() {
        let server = Server::instance().await;

        let response = server.get("/quick/1").await;
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

    #[tokio::test(flavor = "multi_thread")]
    async fn occurrence() {
        let server = Server::instance().await;

        let response = server.get("/occurrence/1").await;
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

    #[tokio::test(flavor = "multi_thread")]
    async fn list() {
        let server = Server::instance().await;

        let response = server.get("/skull").await;
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
