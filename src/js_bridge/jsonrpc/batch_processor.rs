use crate::db_bridge::DbPool;
use crate::js_bridge::executor::{RuntimeConfig, ScriptExecutor};
use crate::js_bridge::models::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::js_bridge::jsonrpc::request_validator::RequestValidator;
use std::collections::HashMap;

/// 批量请求处理器 - 单一职责：处理批量JSON-RPC请求
pub struct BatchProcessor;

impl BatchProcessor {
    /// 处理批量请求
    pub async fn process_batch(
        requests: Vec<JsonRpcRequest>,
        pool: DbPool,
        headers: HashMap<String, String>,
    ) -> Vec<JsonRpcResponse> {
        // 并行处理所有请求
        let futures = requests
            .into_iter()
            .map(|req| Self::process_single(req, pool.clone(), headers.clone()));

        futures::future::join_all(futures).await
    }

    /// 处理单个请求（供批量处理使用）
    pub async fn process_single(
        json_req: JsonRpcRequest,
        pool: DbPool,
        headers: HashMap<String, String>,
    ) -> JsonRpcResponse {
        let request_id = json_req.id.clone();

        // 验证请求
        if let Err(err) = RequestValidator::validate_request(&json_req) {
            return JsonRpcResponse::error(err, request_id);
        }

        // 验证脚本存在
        if let Err(err) = RequestValidator::validate_script_exists(&json_req.method) {
            return JsonRpcResponse::error(err, request_id);
        }

        // 执行脚本
        Self::execute_script(json_req, pool, headers).await
    }

    /// 执行脚本
    async fn execute_script(
        json_req: JsonRpcRequest,
        pool: DbPool,
        headers: HashMap<String, String>,
    ) -> JsonRpcResponse {
        let request_id = json_req.id.clone();
        let script_path = format!("./scripts/{}.js", json_req.method);

        let params_json = json_req.params.unwrap_or(serde_json::Value::Null);
        let body_str = params_json.to_string();

        let js_req = crate::js_bridge::models::JsRequest::new(
            "JSON-RPC".to_string(),
            format!("/rpc/{}", json_req.method),
            headers,
            body_str,
        );

        let config = RuntimeConfig {
            script_path,
            request: js_req,
            db_pool: pool,
        };

        let js_response = ScriptExecutor::execute(config).await;

        Self::build_result(js_response, request_id)
    }

    /// 构建结果
    fn build_result(js_response: crate::js_bridge::models::JsResponse, request_id: Option<serde_json::Value>) -> JsonRpcResponse {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_process_batch() {
        let pool = crate::db_bridge::establish_connection_pool();
        let headers = HashMap::new();

        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "add".to_string(),
                params: Some(json!({"a": 1, "b": 2})),
                id: Some(json!(1)),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "multiply".to_string(),
                params: Some(json!({"a": 3, "b": 4})),
                id: Some(json!(2)),
            },
        ];

        let responses = BatchProcessor::process_batch(requests, pool, headers).await;
        assert_eq!(responses.len(), 2);
        assert!(responses[0].result.is_some());
        assert!(responses[1].result.is_some());
    }

    #[tokio::test]
    async fn test_process_batch_with_errors() {
        let pool = crate::db_bridge::establish_connection_pool();
        let headers = HashMap::new();

        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "add".to_string(),
                params: Some(json!({"a": 1, "b": 2})),
                id: Some(json!(1)),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "nonexistent".to_string(),
                params: Some(json!({})),
                id: Some(json!(2)),
            },
        ];

        let responses = BatchProcessor::process_batch(requests, pool, headers).await;
        assert_eq!(responses.len(), 2);
        assert!(responses[0].result.is_some());
        assert!(responses[1].error.is_some());
    }
}