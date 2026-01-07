use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use crate::test_utils::get_test_pool;
use super::super::handle_js_script;

#[tokio::test]
async fn test_js_request_methods() {
    let pool = crate::db_bridge::get_test_pool();
    let req = Request::builder()
        .method("POST")
        .uri("/js/test_request.js")
        .header("X-Test", "value")
        .body(Body::from("hello world"))
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("test_request.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["method"], "POST");
    assert_eq!(json["path"], "/js/test_request.js");
    assert_eq!(json["headers"]["x-test"], "value");
    assert_eq!(json["body"], "hello world");
    assert_eq!(json["x_test"], "value");
    assert!(json["non_existent"].is_null());
}

#[tokio::test]
async fn test_js_async_with_threadpool() {
    let pool = crate::db_bridge::get_test_pool();
    let req = Request::builder()
        .method("GET")
        .uri("/js/async_test.js")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("async_test.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["message"], "Hello from async JS");
}

#[tokio::test]
async fn test_script_not_found() {
    let pool = crate::db_bridge::get_test_pool();
    let req = Request::builder()
        .method("GET")
        .uri("/js/non_existent.js")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("non_existent.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_ts_transpilation() {
    let pool = crate::db_bridge::get_test_pool();
    let ts_code = r#"
            interface User { name: string; }
            const user: User = { name: "TS User" };
            const res = {
                status: 200,
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ message: `Hello from ${user.name}` })
            };
            Deno.core.ops.op_send_response(res);
        "#;
    std::fs::create_dir_all("./scripts").unwrap();
    std::fs::write("./scripts/test_ts_transpile.ts", ts_code).unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/js/test_ts_transpile.ts")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("test_ts_transpile.ts".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&body_bytes)).unwrap();
    assert_eq!(json["message"], "Hello from TS User");

    let _ = std::fs::remove_file("./scripts/test_ts_transpile.ts");
}

#[tokio::test]
async fn test_js_syntax_error_handling() {
    let pool = crate::db_bridge::get_test_pool();
    let bad_code = "this is not javascript";
    std::fs::write("./scripts/bad_syntax.js", bad_code).unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/js/bad_syntax.js")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("bad_syntax.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let _ = std::fs::remove_file("./scripts/bad_syntax.js");
}

#[tokio::test]
async fn test_js_sql_operations() {
    let pool = crate::db_bridge::get_test_pool();
    let req = Request::builder()
        .method("GET")
        .uri("/js/db_test.js")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("db_test.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["setup"], "ok");
    assert_eq!(json["inserted"], 1);
    assert_eq!(json["queried"][0]["name"], "js_user");
    assert_eq!(json["queried"][0]["res2"], "js@example.com");
    assert_eq!(json["updated_email"], "updated@example.com");
    assert_eq!(json["deleted"], "ok");
}

#[tokio::test]
async fn test_js_sql_dynamic_row() {
    let pool = crate::db_bridge::get_test_pool();
    let req = Request::builder()
        .method("GET")
        .uri("/js/db_dynamic_test.js")
        .body(Body::empty())
        .unwrap();

    let response = handle_js_script(State(pool.clone()), Path("db_dynamic_test.js".to_string()), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["count"], 2);
    assert_eq!(json["first_row"]["name"], "Alice");
    assert_eq!(json["first_row"]["age"], 30);
    assert_eq!(json["first_row"]["metadata"], "developer");
    assert_eq!(json["subset"][1]["name"], "Bob");
    assert!(json["subset"][1]["age"].is_null());
}