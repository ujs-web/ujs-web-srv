use crate::db_bridge::DbPool;
use crate::js_bridge::jsonrpc::batch_processor::BatchProcessor;
use crate::js_bridge::jsonrpc::request_parser::{JsonRpcRequestType, RequestParser};
use crate::js_bridge::jsonrpc::request_validator::RequestValidator;
use crate::js_bridge::jsonrpc::response_builder::ResponseBuilder;
use axum::extract::{Request, State};
use axum::response::IntoResponse;

/// JSON-RPC处理器 - 单一职责：协调请求解析、验证、处理和响应构建
pub async fn handle_json_rpc(
    State(pool): State<DbPool>,
    req: Request,
) -> impl IntoResponse {
    // 解析HTTP请求
    let parsed_req = match RequestParser::parse_http_request(req).await {
        Ok(req) => req,
        Err(err) => return ResponseBuilder::build_error_response(err),
    };

    // 解析JSON-RPC请求
    let json_rpc_req = match RequestParser::parse_json_rpc_request(&parsed_req.body) {
        Ok(req) => req,
        Err(err) => return ResponseBuilder::build_error_response(err),
    };

    // 根据请求类型处理
    match json_rpc_req {
        JsonRpcRequestType::Single(req) => {
            handle_single_request(req, pool, parsed_req.headers)
                .await
                .into_response()
        }
        JsonRpcRequestType::Batch(reqs) => {
            handle_batch_request(reqs, pool, parsed_req.headers)
                .await
                .into_response()
        }
    }
}

/// 处理单个请求
async fn handle_single_request(
    req: crate::js_bridge::models::JsonRpcRequest,
    pool: DbPool,
    headers: std::collections::HashMap<String, String>,
) -> impl IntoResponse {
    // 验证请求
    if let Err(err) = RequestValidator::validate_request(&req) {
        return ResponseBuilder::build_response(crate::js_bridge::models::JsonRpcResponse::error(
            err,
            req.id,
        ));
    }

    // 验证脚本存在
    if let Err(err) = RequestValidator::validate_script_exists(&req.method) {
        return ResponseBuilder::build_response(crate::js_bridge::models::JsonRpcResponse::error(
            err,
            req.id,
        ));
    }

    // 处理请求
    let response = BatchProcessor::process_single(req, pool, headers).await;
    ResponseBuilder::build_response(response)
}

/// 处理批量请求
async fn handle_batch_request(
    reqs: Vec<crate::js_bridge::models::JsonRpcRequest>,
    pool: DbPool,
    headers: std::collections::HashMap<String, String>,
) -> impl IntoResponse {
    // 验证批量请求不为空
    if let Err(err) = RequestValidator::validate_batch_not_empty(&reqs) {
        return ResponseBuilder::build_error_response(err);
    }

    // 处理批量请求
    let responses = BatchProcessor::process_batch(reqs, pool, headers).await;
    ResponseBuilder::build_batch_response(responses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Method, Request};
    use serde_json::json;

    fn create_test_request(body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_handle_single_request() {
        let pool = crate::db_bridge::establish_connection_pool();
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "add",
            "params": {"a": 5, "b": 3},
            "id": 1
        });

        let request = create_test_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_batch_request() {
        let pool = crate::db_bridge::establish_connection_pool();
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
            }
        ]);

        let request = create_test_request(request_body);
        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_invalid_content_type() {
        let pool = crate::db_bridge::establish_connection_pool();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/rpc")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("{}"))
            .unwrap();

        let response = handle_json_rpc(State(pool), request)
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}