use crate::js_bridge::models::{JsonRpcError, JsonRpcResponse};
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// 响应构建器 - 单一职责：构建JSON-RPC响应
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// 构建单个响应
    pub fn build_response(resp: JsonRpcResponse) -> Response {
        resp.into_response()
    }

    /// 构建批量响应
    pub fn build_batch_response(responses: Vec<JsonRpcResponse>) -> Response {
        // 过滤掉通知请求（没有 id 的请求）
        let filtered_responses: Vec<JsonRpcResponse> = responses
            .into_iter()
            .filter(|r| r.id.is_some())
            .collect();

        // 如果所有请求都是通知，返回空数组
        if filtered_responses.is_empty() {
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from("[]"))
                .unwrap();
        }

        let body = serde_json::to_string(&filtered_responses).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    /// 构建错误响应
    pub fn build_error_response(error: JsonRpcError) -> Response {
        JsonRpcResponse::error(error, None).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_response() {
        let resp = JsonRpcResponse::success(json!({"result": 8}), Some(json!(1)));
        let response = ResponseBuilder::build_response(resp);
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_build_batch_response() {
        let responses = vec![
            JsonRpcResponse::success(json!({"result": 3}), Some(json!(1))),
            JsonRpcResponse::success(json!({"result": 12}), Some(json!(2))),
        ];
        let response = ResponseBuilder::build_batch_response(responses);
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_build_batch_response_with_notifications() {
        let responses = vec![
            JsonRpcResponse::success(json!({"result": 3}), Some(json!(1))),
            JsonRpcResponse::success(json!({"result": 12}), None), // 通知
            JsonRpcResponse::success(json!({"result": 30}), Some(json!(2))),
        ];
        let response = ResponseBuilder::build_batch_response(responses);
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed.len(), 2); // 通知被过滤掉
    }

    #[tokio::test]
    async fn test_build_batch_response_all_notifications() {
        let responses = vec![
            JsonRpcResponse::success(json!({"result": 3}), None),
            JsonRpcResponse::success(json!({"result": 12}), None),
        ];
        let response = ResponseBuilder::build_batch_response(responses);
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], b"[]");
    }

    #[test]
    fn test_build_error_response() {
        let error = JsonRpcError::invalid_request("test error");
        let response = ResponseBuilder::build_error_response(error);
        assert_eq!(response.status(), StatusCode::OK);
    }
}