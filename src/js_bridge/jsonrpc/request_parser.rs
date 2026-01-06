use crate::js_bridge::models::{JsonRpcError, JsonRpcRequest};
use axum::body::Body;
use axum::extract::Request;
use axum::http::header;
use std::collections::HashMap;

/// 请求解析器 - 单一职责：解析HTTP请求并提取JSON-RPC数据
pub struct RequestParser;

impl RequestParser {
    /// 解析HTTP请求，提取headers和body
    pub async fn parse_http_request(req: Request<Body>) -> Result<ParsedHttpRequest, JsonRpcError> {
        let (parts, body) = req.into_parts();

        // 验证Content-Type
        let content_type = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("application/json") {
            return Err(JsonRpcError::invalid_request(
                "Content-Type must be application/json",
            ));
        }

        // 读取body
        let body_bytes = axum::body::to_bytes(body, 1024 * 1024).await
            .map_err(|e| JsonRpcError::invalid_request(&format!("Failed to read body: {}", e)))?;

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        // 提取headers
        let headers = Self::extract_headers(&parts.headers);

        Ok(ParsedHttpRequest {
            body: body_str,
            headers,
        })
    }

    /// 解析JSON-RPC请求（单个或批量）
    pub fn parse_json_rpc_request(body: &str) -> Result<JsonRpcRequestType, JsonRpcError> {
        // 尝试解析为单个请求
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(body) {
            return Ok(JsonRpcRequestType::Single(req));
        }

        // 尝试解析为批量请求
        if let Ok(batch) = serde_json::from_str::<Vec<JsonRpcRequest>>(body) {
            return Ok(JsonRpcRequestType::Batch(batch));
        }

        // 解析失败
        Err(JsonRpcError::parse_error("Failed to parse JSON-RPC request"))
    }

    /// 从HTTP headers提取HashMap
    fn extract_headers(headers: &axum::http::HeaderMap) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (name, value) in headers.iter() {
            if let Ok(v) = value.to_str() {
                result.insert(name.to_string(), v.to_string());
            }
        }
        result
    }
}

/// 解析后的HTTP请求数据
#[derive(Debug)]
pub struct ParsedHttpRequest {
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// JSON-RPC请求类型（单个或批量）
#[derive(Debug)]
pub enum JsonRpcRequestType {
    Single(JsonRpcRequest),
    Batch(Vec<JsonRpcRequest>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Method, Request};

    fn create_test_request(body: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_parse_http_request_success() {
        let body = r#"{"jsonrpc":"2.0","method":"add","params":{},"id":1}"#;
        let req = create_test_request(body);

        let result = RequestParser::parse_http_request(req).await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.body, body);
        assert!(parsed.headers.contains_key("content-type"));
    }

    #[tokio::test]
    async fn test_parse_invalid_content_type() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("{}"))
            .unwrap();

        let result = RequestParser::parse_http_request(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32600);
    }

    #[test]
    fn test_parse_single_request() {
        let body = r#"{"jsonrpc":"2.0","method":"add","params":{},"id":1}"#;
        let result = RequestParser::parse_json_rpc_request(body);
        assert!(result.is_ok());
        match result.unwrap() {
            JsonRpcRequestType::Single(req) => {
                assert_eq!(req.method, "add");
            }
            _ => panic!("Expected single request"),
        }
    }

    #[test]
    fn test_parse_batch_request() {
        let body = r#"[
            {"jsonrpc":"2.0","method":"add","params":{},"id":1},
            {"jsonrpc":"2.0","method":"multiply","params":{},"id":2}
        ]"#;
        let result = RequestParser::parse_json_rpc_request(body);
        assert!(result.is_ok());
        match result.unwrap() {
            JsonRpcRequestType::Batch(reqs) => {
                assert_eq!(reqs.len(), 2);
            }
            _ => panic!("Expected batch request"),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let body = r#"invalid json"#;
        let result = RequestParser::parse_json_rpc_request(body);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32700);
    }
}