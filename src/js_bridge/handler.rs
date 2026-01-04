use crate::db_bridge::DbPool;
use crate::js_bridge::executor::{RuntimeConfig, ScriptExecutor};
use crate::js_bridge::models::{JsRequest, JsResponse};
use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
};
use std::collections::HashMap;

pub async fn handle_js_script(
    State(pool): State<DbPool>,
    Path(script_name): Path<String>,
    req: Request,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let method = parts.method.to_string();
    let path = parts.uri.path().to_string();
    let mut headers = HashMap::new();
    for (name, value) in parts.headers.iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.to_string(), v.to_string());
        }
    }

    let body_bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return JsResponse::internal_error("Failed to read body").into_response();
        }
    };
    let body_str = String::from_utf8_lossy(&body_bytes).into_owned();

    let js_req = JsRequest::new(method, path, headers, body_str);

    let config = RuntimeConfig {
        script_path: format!("./scripts/{}", script_name),
        request: js_req,
        db_pool: pool,
    };

    ScriptExecutor::execute(config).await.into_response()
}
