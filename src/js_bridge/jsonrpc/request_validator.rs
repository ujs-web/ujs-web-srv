use crate::js_bridge::models::{JsonRpcError, JsonRpcRequest};

/// 请求验证器 - 单一职责：验证JSON-RPC请求的格式和内容
pub struct RequestValidator;

impl RequestValidator {
    /// 验证JSON-RPC请求的基本格式
    pub fn validate_request(req: &JsonRpcRequest) -> Result<(), JsonRpcError> {
        // 验证jsonrpc版本
        if req.jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request("jsonrpc version must be 2.0"));
        }

        // 验证方法名
        if req.method.is_empty() {
            return Err(JsonRpcError::invalid_request("method is required"));
        }

        Ok(())
    }

    /// 验证脚本文件是否存在
    pub fn validate_script_exists(method: &str) -> Result<(), JsonRpcError> {
        let script_path = format!("./scripts/{}.js", method);
        if !std::path::Path::new(&script_path).exists() {
            return Err(JsonRpcError::method_not_found(method));
        }
        Ok(())
    }

    /// 验证批量请求不为空
    pub fn validate_batch_not_empty(requests: &[JsonRpcRequest]) -> Result<(), JsonRpcError> {
        if requests.is_empty() {
            return Err(JsonRpcError::invalid_request("Batch request cannot be empty"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_request(method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(json!({})),
            id: Some(json!(1)),
        }
    }

    #[test]
    fn test_validate_request_success() {
        let req = create_test_request("add");
        assert!(RequestValidator::validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_invalid_jsonrpc_version() {
        let mut req = create_test_request("add");
        req.jsonrpc = "1.0".to_string();
        let result = RequestValidator::validate_request(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32600);
    }

    #[test]
    fn test_validate_empty_method() {
        let req = create_test_request("");
        let result = RequestValidator::validate_request(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32600);
    }

    #[test]
    fn test_validate_script_exists() {
        // 假设add.js存在
        let result = RequestValidator::validate_script_exists("add");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_script_not_exists() {
        let result = RequestValidator::validate_script_exists("nonexistent");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32601);
    }

    #[test]
    fn test_validate_batch_not_empty() {
        let requests = vec![create_test_request("add")];
        assert!(RequestValidator::validate_batch_not_empty(&requests).is_ok());
    }

    #[test]
    fn test_validate_batch_empty() {
        let requests: Vec<JsonRpcRequest> = vec![];
        let result = RequestValidator::validate_batch_not_empty(&requests);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32600);
    }
}