// JSON-RPC处理器 - 已重构为模块化结构
// 为了向后兼容，保留此文件作为入口点
// 实际实现已迁移到 jsonrpc/ 模块

pub use crate::js_bridge::jsonrpc::handle_json_rpc;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::State,
        http::{header, Method, Request, StatusCode},
        response::IntoResponse,
    };
    use serde_json::json;

    fn create_test_pool() -> crate::db_bridge::DbPool {
        crate::db_bridge::establish_connection_pool()
    }

    fn create_json_rpc_request(body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_single_request_success() {
        let pool = create_test_pool();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "add",
            "params": {"a": 5, "b": 3},
            "id": 1
        });

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response.jsonrpc, "2.0");
        assert!(json_response.result.is_some());
        assert!(json_response.error.is_none());
        assert_eq!(json_response.id, Some(json!(1)));

        let result = json_response.result.as_ref().unwrap();
        assert_eq!(result["result"], 8);
    }

    #[tokio::test]
    async fn test_single_request_multiply() {
        let pool = create_test_pool();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "multiply",
            "params": {"a": 4, "b": 7},
            "id": 2
        });

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response.jsonrpc, "2.0");
        assert!(json_response.result.is_some());
        assert!(json_response.error.is_none());

        let result = json_response.result.as_ref().unwrap();
        assert_eq!(result["result"], 28);
    }

    #[tokio::test]
    async fn test_batch_request_success() {
        let pool = create_test_pool();
        let request_body = json!([
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 1, "b": 2},
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "multiply",
                "params": {"a": 3, "b": 4},
                "id": 2
            },
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 10, "b": 20},
                "id": 3
            }
        ]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let responses: Vec<crate::js_bridge::models::JsonRpcResponse> =
            serde_json::from_slice(&body).unwrap();

        assert_eq!(responses.len(), 3);

        // 验证第一个响应
        assert_eq!(responses[0].jsonrpc, "2.0");
        assert_eq!(responses[0].id, Some(json!(1)));
        assert_eq!(responses[0].result.as_ref().unwrap()["result"], 3);

        // 验证第二个响应
        assert_eq!(responses[1].jsonrpc, "2.0");
        assert_eq!(responses[1].id, Some(json!(2)));
        assert_eq!(responses[1].result.as_ref().unwrap()["result"], 12);

        // 验证第三个响应
        assert_eq!(responses[2].jsonrpc, "2.0");
        assert_eq!(responses[2].id, Some(json!(3)));
        assert_eq!(responses[2].result.as_ref().unwrap()["result"], 30);
    }

    #[tokio::test]
    async fn test_batch_request_with_notifications() {
        let pool = create_test_pool();
        let request_body = json!([
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 1, "b": 2},
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "multiply",
                "params": {"a": 3, "b": 4}
            },
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 5, "b": 6},
                "id": 2
            }
        ]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let responses: Vec<crate::js_bridge::models::JsonRpcResponse> =
            serde_json::from_slice(&body).unwrap();

        // 应该只返回两个响应（通知请求被过滤掉）
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].id, Some(json!(1)));
        assert_eq!(responses[1].id, Some(json!(2)));
    }

    #[tokio::test]
    async fn test_all_notifications() {
        let pool = create_test_pool();
        let request_body = json!([
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 1, "b": 2}
            },
            {
                "jsonrpc": "2.0",
                "method": "multiply",
                "params": {"a": 3, "b": 4}
            }
        ]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"[]");
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let pool = create_test_pool();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "nonexistent",
            "params": {},
            "id": 1
        });

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[tokio::test]
    async fn test_invalid_jsonrpc_version() {
        let pool = create_test_pool();
        let request_body = json!({
            "jsonrpc": "1.0",
            "method": "add",
            "params": {"a": 1, "b": 2},
            "id": 1
        });

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[tokio::test]
    async fn test_empty_method() {
        let pool = create_test_pool();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "",
            "params": {},
            "id": 1
        });

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
    }

    #[tokio::test]
    async fn test_invalid_content_type() {
        let pool = create_test_pool();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("{}"))
            .unwrap();

        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
    }

    #[tokio::test]
    async fn test_empty_batch_request() {
        let pool = create_test_pool();
        let request_body = json!([]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: crate::js_bridge::models::JsonRpcResponse =
            serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert!(error.data.unwrap().as_str().unwrap().contains("empty"));
    }

    #[tokio::test]
    async fn test_batch_with_mixed_success_and_errors() {
        let pool = create_test_pool();
        let request_body = json!([
            {
                "jsonrpc": "2.0",
                "method": "add",
                "params": {"a": 1, "b": 2},
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "nonexistent",
                "params": {},
                "id": 2
            },
            {
                "jsonrpc": "2.0",
                "method": "multiply",
                "params": {"a": 3, "b": 4},
                "id": 3
            }
        ]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let responses: Vec<crate::js_bridge::models::JsonRpcResponse> =
            serde_json::from_slice(&body).unwrap();

        assert_eq!(responses.len(), 3);

        // 第一个成功
        assert!(responses[0].result.is_some());
        assert_eq!(responses[0].result.as_ref().unwrap()["result"], 3);

        // 第二个失败
        assert!(responses[1].error.is_some());
        assert_eq!(responses[1].error.as_ref().unwrap().code, -32601);

        // 第三个成功
        assert!(responses[2].result.is_some());
        assert_eq!(responses[2].result.as_ref().unwrap()["result"], 12);
    }
}