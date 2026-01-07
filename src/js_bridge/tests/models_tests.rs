use std::collections::HashMap;
use axum::response::IntoResponse;
use super::super::models::{JsRequest, JsResponse};

#[tokio::test]
async fn test_request_getters() {
    let mut headers = HashMap::new();
    headers.insert("X-Key".to_string(), "Value".to_string());
    let req = JsRequest {
        method: "GET".to_string(),
        path: "/test".to_string(),
        headers: headers.clone(),
        body: "body".to_string(),
    };

    assert_eq!(req.get_method(), "GET");
    assert_eq!(req.get_path(), "/test");
    assert_eq!(req.get_headers(), headers);
    assert_eq!(req.get_body(), "body");
    assert_eq!(req.get_header("X-Key"), Some("Value".to_string()));
    assert_eq!(req.get_header("Non-Existent"), None);
}

#[tokio::test]
async fn test_js_response_into_response() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom".to_string(), "Value".to_string());
    let js_res = JsResponse {
        status: 201,
        headers,
        body: "created".to_string(),
    };

    let res = js_res.into_response();
    assert_eq!(res.status(), axum::http::StatusCode::CREATED);
    assert_eq!(res.headers().get("X-Custom").unwrap(), "Value");
}