use hyper::StatusCode;

use crate::{
    check_async as check,
    client::Client,
    server,
    utils::{
        EMPTY_USER, USER_HEADER, build_occurrence_payload, build_skull_payload, eq, extract_body,
    },
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
            test!(list),
            test!(list_empty),
            test!(create),
            test!(create_constraint),
            test!(create_conflict),
            test!(create_bad_payload),
            test!(search),
            test!(search_not_found),
            test!(search_limit),
            test!(update),
            test!(update_no_change),
            test!(update_not_found),
            test!(update_constraint),
            test!(update_conflict),
            test!(update_bad_payload),
            test!(delete),
            test!(delete_not_found),
            test!(delete_rejected),
        ]
    })
}

async fn missing_user(client: Client) {
    let response = client
        .get_with("skull", |r| {
            r.headers_mut().remove(USER_HEADER);
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, ""));
}

async fn unknown_user(client: Client) {
    let response = client
        .get_with("skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, "unknown".try_into().unwrap());
        })
        .await;

    check!(eq(response, StatusCode::FORBIDDEN, ""));
}

async fn method_not_allowed(client: Client) {
    let response = client
        .get_with("skull", |r| *r.method_mut() = hyper::Method::PUT)
        .await;

    check!(eq(response, StatusCode::METHOD_NOT_ALLOWED, ""));
}

async fn not_found(client: Client) {
    let response = client.get("bloink").await;

    check!(eq(response, StatusCode::NOT_FOUND, ""));
}

async fn list(client: Client) {
    let response = client.get("skull").await;

    check!(eq(response, StatusCode::OK, build_skull_payload([1, 2, 3])));
}

async fn list_empty(client: Client) {
    let response = client
        .get_with("skull", |r| {
            r.headers_mut()
                .insert(USER_HEADER, EMPTY_USER.try_into().unwrap());
        })
        .await;

    check!(eq(response, StatusCode::OK, build_skull_payload([])));
}

async fn create(client: Client) {
    let response = client
        .post(
            "skull",
            r#"{
                "name": "skull27",
                "color": 27,
                "icon": "icon27",
                "price": 0.27
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::CREATED,
        "{\"change\":\"created\"}"
    ));
}

async fn create_constraint(client: Client) {
    let response = client
        .post(
            "occurrence",
            r#"{
                "items": [{
                    "skull": 27,
                    "amount": 27,
                    "millis": 0
                }]
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"referenced ID does not exist\"}}"
    ));
}

async fn create_conflict(client: Client) {
    let response = client
        .post(
            "skull",
            r#"{
                "name": "skull1",
                "color": 1,
                "icon": "icon1",
                "price": 0.1
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"entry already exists: UNIQUE constraint failed: skulls.icon\"}}"
    ));
}

async fn create_bad_payload(client: Client) {
    let response = client.post("occurrence", r#"{"bloink": 27}"#).await;

    check!(eq(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Failed to deserialize the JSON body into the target type: missing field `items` at line 1 column 14"
    ));
}

async fn create_empty(client: Client) {
    let response = client.post("occurrence", "{\"items\":[]}").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"no changes specified\"}}"
    ));
}

async fn search(client: Client) {
    let response = client.get("occurrence?skulls=2").await;

    check!(eq(response, StatusCode::OK, build_occurrence_payload([2])));
}

async fn search_not_found(client: Client) {
    let response = client.get("occurrence?skulls=27").await;

    check!(eq(response, StatusCode::OK, "{\"occurrences\":[]}"));
}

async fn search_limit(client: Client) {
    for i in 0..5 {
        let response = client.get(format!("occurrence?limit={i}")).await;

        let payload = match i {
            0 => build_occurrence_payload([]),
            1 => build_occurrence_payload([3]),
            2 => build_occurrence_payload([3, 2]),
            _ => build_occurrence_payload([3, 2, 1]),
        };
        check!(eq(response, StatusCode::OK, payload));
    }
}

async fn update(client: Client) {
    let response = client.get("occurrence").await;
    let original = extract_body(response).await;

    let response = client
        .patch(
            "occurrence",
            r#"{
                "id": 3,
                "skull": { "set": 3 },
                "amount": { "set": 27 }
            }"#,
        )
        .await;

    check!(eq(response, StatusCode::NO_CONTENT, ""));

    let response = client.get("occurrence").await;
    let modified = original.replace("\"amount\":3.0", "\"amount\":27.0");
    check!(eq(response, StatusCode::OK, modified));
}

async fn update_no_change(client: Client) {
    let response = client.patch("occurrence", "{\"id\": 3}").await;
    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"no changes specified\"}}"
    ));
}

async fn update_not_found(client: Client) {
    let response = client
        .patch(
            "occurrence",
            r#"{
                "id": 27,
                "skull": { "set": 3 },
                "amount": { "set": 27 }
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::NOT_FOUND,
        "{\"error\":{\"kind\":\"NotFound\",\"message\":\"entry not found for `27`\"}}"
    ));
}

async fn update_constraint(client: Client) {
    let response = client
        .patch(
            "occurrence",
            r#"{
                "id": 3,
                "skull": { "set": 27 }
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"referenced ID does not exist\"}}"
    ));
}

async fn update_conflict(client: Client) {
    let response = client
        .patch(
            "skull",
            r#"{
                "id": 2,
                "icon": { "set": "icon1" }
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"entry already exists: UNIQUE constraint failed: skulls.icon\"}}"
    ));
}

async fn update_bad_payload(client: Client) {
    let response = client
        .patch(
            "occurrence",
            r#"{
                "id": 1,
                "amount": 2
            }"#,
        )
        .await;

    check!(eq(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Failed to deserialize the JSON body into the target type: amount: invalid type: integer `2`, expected struct Setter at line 3 column 27"
    ));
}

async fn delete(client: Client) {
    let response = client.delete("occurrence", "{\"id\":3}").await;
    check!(eq(response, StatusCode::NO_CONTENT, ""));

    let response = client.get("occurrence").await;
    check!(eq(
        response,
        StatusCode::OK,
        build_occurrence_payload([2, 1])
    ));
}

async fn delete_not_found(client: Client) {
    let response = client.delete("occurrence", "{\"id\":27}").await;

    check!(eq(
        response,
        StatusCode::NOT_FOUND,
        "{\"error\":{\"kind\":\"NotFound\",\"message\":\"entry not found for `27`\"}}"
    ));
}

async fn delete_rejected(client: Client) {
    let response = client.delete("skull", "{\"id\":1}").await;

    check!(eq(
        response,
        StatusCode::BAD_REQUEST,
        "{\"error\":{\"kind\":\"BadRequest\",\"message\":\"entry fails constraint check: FOREIGN KEY constraint failed\"}}"
    ));
}
