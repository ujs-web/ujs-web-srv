use axum::{
    body::Body,
    http::{HeaderName, HeaderValue},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}

impl JsRequest {
    pub fn new(method: String, path: String, headers: HashMap<String, String>, body: String) -> Self {
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
