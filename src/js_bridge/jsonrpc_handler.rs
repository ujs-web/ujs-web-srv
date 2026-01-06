use crate::db_bridge::DbPool;
use crate::js_bridge::executor::{RuntimeConfig, ScriptExecutor};
use crate::js_bridge::models::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use axum::{
    extract::{Request, State},
    response::IntoResponse,
};
use std::collections::HashMap;

async fn process_single_request(
    json_req: JsonRpcRequest,
    pool: DbPool,
    headers: HashMap<String, String>,
) -> JsonRpcResponse {
    let request_id = json_req.id.clone();

    if json_req.jsonrpc != "2.0" {
        return JsonRpcResponse::error(
            JsonRpcError::invalid_request("jsonrpc version must be 2.0"),
            request_id,
        );
    }

    if json_req.method.is_empty() {
        return JsonRpcResponse::error(
            JsonRpcError::invalid_request("method is required"),
            request_id,
        );
    }

    let script_path = format!("./scripts/{}.js", json_req.method);

    if !std::path::Path::new(&script_path).exists() {
        return JsonRpcResponse::error(
            JsonRpcError::method_not_found(&json_req.method),
            request_id,
        );
    }

    let params_json = json_req.params.unwrap_or(serde_json::Value::Null);
    let body_str = params_json.to_string();

    let js_req = crate::js_bridge::models::JsRequest::new(
        "JSON-RPC".to_string(),
        format!("/rpc/{}", json_req.method),
        headers,
        body_str,
    );

    let config = RuntimeConfig {
        script_path: script_path.to_owned(),
        request: js_req,
        db_pool: pool,
    };

    let js_response = ScriptExecutor::execute(config).await;

    if js_response.status == 200 {
        let result: serde_json::Value = match serde_json::from_str(&js_response.body) {
            Ok(v) => v,
            Err(_) => serde_json::json!(js_response.body),
        };
        JsonRpcResponse::success(result, request_id)
    } else {
        JsonRpcResponse::error(
            JsonRpcError::internal_error(&js_response.body),
            request_id,
        )
    }
}

pub async fn handle_json_rpc(
    State(pool): State<DbPool>,
    req: Request,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("application/json") {
        return JsonRpcResponse::error(
            JsonRpcError::invalid_request("Content-Type must be application/json"),
            None,
        )
        .into_response();
    }

    let body_bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return JsonRpcResponse::error(
                JsonRpcError::invalid_request(&format!("Failed to read body: {}", e)),
                None,
            )
            .into_response();
        }
    };

    let body_str = String::from_utf8_lossy(&body_bytes);

    // 提取 headers
    let mut headers = HashMap::new();
    for (name, value) in parts.headers.iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.to_string(), v.to_string());
        }
    }

    // 尝试解析为单个请求
    if let Ok(json_req) = serde_json::from_str::<JsonRpcRequest>(&body_str) {
        let response = process_single_request(json_req, pool, headers).await;
        return response.into_response();
    }

    // 尝试解析为批量请求
    if let Ok(json_batch) = serde_json::from_str::<Vec<JsonRpcRequest>>(&body_str) {
        if json_batch.is_empty() {
            return JsonRpcResponse::error(
                JsonRpcError::invalid_request("Batch request cannot be empty"),
                None,
            )
            .into_response();
        }

        // 并行处理所有请求
        let futures = json_batch
            .into_iter()
            .map(|req| process_single_request(req, pool.clone(), headers.clone()));

        let responses: Vec<JsonRpcResponse> = futures::future::join_all(futures).await;

        // 过滤掉通知请求（没有 id 的请求）
        let filtered_responses: Vec<JsonRpcResponse> = responses
            .into_iter()
            .filter(|r| r.id.is_some())
            .collect();

        // 如果所有请求都是通知，返回空响应
        if filtered_responses.is_empty() {
            return axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from("[]"))
                .unwrap()
                .into_response();
        }

        let body = serde_json::to_string(&filtered_responses).unwrap();
        return axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
            .into_response();
    }

    // 既不是单个请求也不是批量请求，返回解析错误
    JsonRpcResponse::error(
        JsonRpcError::parse_error("Failed to parse JSON-RPC request"),
        None,
    )
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
    };

    fn create_test_pool() -> DbPool {
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
        let request_body = serde_json::json!({
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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response.jsonrpc, "2.0");
        assert!(json_response.result.is_some());
        assert!(json_response.error.is_none());
        assert_eq!(json_response.id, Some(serde_json::json!(1)));

        let result = json_response.result.as_ref().unwrap();
        assert_eq!(result["result"], 8);
    }

    #[tokio::test]
    async fn test_single_request_multiply() {
        let pool = create_test_pool();
        let request_body = serde_json::json!({
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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json_response.jsonrpc, "2.0");
        assert!(json_response.result.is_some());
        assert!(json_response.error.is_none());

        let result = json_response.result.as_ref().unwrap();
        assert_eq!(result["result"], 28);
    }

    #[tokio::test]
    async fn test_batch_request_success() {
        let pool = create_test_pool();
        let request_body = serde_json::json!([
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
        let responses: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();

        assert_eq!(responses.len(), 3);

        // 验证第一个响应
        assert_eq!(responses[0].jsonrpc, "2.0");
        assert_eq!(responses[0].id, Some(serde_json::json!(1)));
        assert_eq!(responses[0].result.as_ref().unwrap()["result"], 3);

        // 验证第二个响应
        assert_eq!(responses[1].jsonrpc, "2.0");
        assert_eq!(responses[1].id, Some(serde_json::json!(2)));
        assert_eq!(responses[1].result.as_ref().unwrap()["result"], 12);

        // 验证第三个响应
        assert_eq!(responses[2].jsonrpc, "2.0");
        assert_eq!(responses[2].id, Some(serde_json::json!(3)));
        assert_eq!(responses[2].result.as_ref().unwrap()["result"], 30);
    }

    #[tokio::test]
    async fn test_batch_request_with_notifications() {
        let pool = create_test_pool();
        let request_body = serde_json::json!([
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
        let responses: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();

        // 应该只返回两个响应（通知请求被过滤掉）
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].id, Some(serde_json::json!(1)));
        assert_eq!(responses[1].id, Some(serde_json::json!(2)));
    }

    #[tokio::test]
    async fn test_all_notifications() {
        let pool = create_test_pool();
        let request_body = serde_json::json!([
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
        let request_body = serde_json::json!({
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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[tokio::test]
    async fn test_invalid_jsonrpc_version() {
        let pool = create_test_pool();
        let request_body = serde_json::json!({
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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[tokio::test]
    async fn test_empty_method() {
        let pool = create_test_pool();
        let request_body = serde_json::json!({
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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

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
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
    }

    #[tokio::test]
    async fn test_empty_batch_request() {
        let pool = create_test_pool();
        let request_body = serde_json::json!([]);

        let request = create_json_rpc_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

        assert!(json_response.error.is_some());
        let error = json_response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert!(error.data.unwrap().as_str().unwrap().contains("empty"));
    }

    #[tokio::test]
    async fn test_batch_with_mixed_success_and_errors() {
        let pool = create_test_pool();
        let request_body = serde_json::json!([
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
        let responses: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();

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
