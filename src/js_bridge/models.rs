use axum::{
    body::Body,
    http::{HeaderName, HeaderValue},
    response::IntoResponse,
};
use deno_core::Resource;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JsRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}
impl Resource for JsRequest {
    fn name(&self) -> Cow<'_, str> {
        "JsRequest".into()
    }
}
impl JsRequest {
    pub fn new(
        method: String,
        path: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
        }
    }

    pub fn get_method(&self) -> String {
        self.method.clone()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn get_body(&self) -> String {
        self.body.clone()
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).cloned()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsResponse {
    pub(crate) status: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}

impl JsResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self::new(500, msg.to_string())
    }

    pub fn not_found(msg: &str) -> Self {
        Self::new(404, msg.to_string())
    }
}

impl IntoResponse for JsResponse {
    fn into_response(self) -> axum::response::Response {
        let mut res_builder = axum::response::Response::builder().status(self.status);

        for (k, v) in self.headers {
            if let (Ok(name), Ok(value)) = (HeaderName::try_from(k), HeaderValue::try_from(v)) {
                res_builder = res_builder.header(name, value);
            }
        }

        res_builder
            .body(Body::from(self.body))
            .unwrap()
            .into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn success(result: serde_json::Value, id: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(error: JsonRpcError, id: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn parse_error(msg: &str) -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(serde_json::json!(msg)),
        }
    }

    pub fn invalid_request(msg: &str) -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::json!(msg)),
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(serde_json::json!(method)),
        }
    }

    /*
        pub fn invalid_params(msg: &str) -> Self {
            Self {
                code: -32602,
                message: "Invalid params".to_string(),
                data: Some(serde_json::json!(msg)),
            }
        }
    */
    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::json!(msg)),
        }
    }
}

impl IntoResponse for JsonRpcResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            serde_json::to_string(&JsonRpcResponse::error(
                JsonRpcError::internal_error("Failed to serialize response"),
                None,
            ))
            .unwrap()
        });

        axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
            .into_response()
    }
}
