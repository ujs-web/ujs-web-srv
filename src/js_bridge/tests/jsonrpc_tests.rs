use axum::body::Body;
use axum::extract::State;
use axum::http::{Request};
use axum::response::IntoResponse;
use crate::js_bridge::jsonrpc::handle_json_rpc;
use super::super::models::{JsonRpcRequest, JsonRpcResponse};

#[tokio::test]
async fn test_json_rpc_add() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();
    let json_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "add".to_string(),
        params: Some(serde_json::json!({"a": 5, "b": 3})),
        id: Some(serde_json::json!(1)),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json_req).unwrap()))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.error.is_none());
    assert_eq!(json_res.id, Some(serde_json::json!(1)));
    assert_eq!(json_res.result.unwrap()["result"], 8);
}

#[tokio::test]
async fn test_json_rpc_multiply() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();
    let json_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "multiply".to_string(),
        params: Some(serde_json::json!({"a": 4, "b": 7})),
        id: Some(serde_json::json!(2)),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json_req).unwrap()))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.error.is_none());
    assert_eq!(json_res.id, Some(serde_json::json!(2)));
    assert_eq!(json_res.result.unwrap()["result"], 28);
}

#[tokio::test]
async fn test_json_rpc_greet() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();
    let json_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "greet".to_string(),
        params: Some(serde_json::json!({"name": "World"})),
        id: Some(serde_json::json!(3)),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json_req).unwrap()))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.error.is_none());
    assert_eq!(json_res.id, Some(serde_json::json!(3)));
    assert_eq!(json_res.result.unwrap()["greeting"], "Hello, World!");
}

#[tokio::test]
async fn test_json_rpc_method_not_found() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();
    let json_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "non_existent_method".to_string(),
        params: None,
        id: Some(serde_json::json!(4)),
    };

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json_req).unwrap()))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.result.is_none());
    assert!(json_res.error.is_some());
    assert_eq!(json_res.id, Some(serde_json::json!(4)));
    assert_eq!(json_res.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_json_rpc_invalid_json() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "application/json")
        .body(Body::from("{invalid json}"))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.result.is_none());
    assert!(json_res.error.is_some());
    assert_eq!(json_res.error.unwrap().code, -32700);
}

#[tokio::test]
async fn test_json_rpc_invalid_request() {
    let pool = crate::db_bridge::get_test_pool();
    let ws_state = crate::websocket::create_websocket_state();

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("Content-Type", "text/plain")
        .body(Body::from("{}"))
        .unwrap();

    let response = handle_json_rpc(State((pool.clone(), ws_state)), req).await;
    let response = response.into_response();

    assert_eq!(response.status(), 200);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);

    let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json_res.jsonrpc, "2.0");
    assert!(json_res.result.is_none());
    assert!(json_res.error.is_some());
    assert_eq!(json_res.error.unwrap().code, -32600);
}