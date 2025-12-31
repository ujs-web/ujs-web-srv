use axum::{
    body::Body,
    extract::{Path, Request},
    http::{HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};
use deno_core::{extension, op2, JsRuntime, OpState, RuntimeOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tokio::sync::oneshot;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

extension!(
    web_runtime,
    ops = [op_log, op_send_response],
);

#[op2(fast)]
fn op_log(#[string] msg: String) {
    println!("[JS Log]: {}", msg);
}

#[op2]
fn op_send_response(state: &mut OpState, #[serde] res: JsResponse) {
    let tx = state.take::<oneshot::Sender<JsResponse>>();
    let _ = tx.send(res);
}

async fn handle_js_script(
    Path(script_path): Path<String>,
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
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read body").into_response(),
    };
    let body_str = String::from_utf8_lossy(&body_bytes).into_owned();

    let js_req = JsRequest {
        method,
        path,
        headers,
        body: body_str,
    };

    // 脚本文件的实际路径
    let full_script_path = format!("./scripts/{}", script_path);
    let script_code = match fs::read_to_string(&full_script_path).await {
        Ok(code) => code,
        Err(_) => return (StatusCode::NOT_FOUND, "Script not found").into_response(),
    };

    let (tx, rx) = oneshot::channel();

    // 在同步代码块中执行 JS，确保 JsRuntime 不跨越 await
    {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            ..Default::default()
        });

        runtime.op_state().borrow_mut().put(tx);

        // 将请求对象注入 JS 环境
        let req_json = serde_json::to_string(&js_req).unwrap();
        let init_code = format!("globalThis.request = {};", req_json);
        runtime.execute_script("<init>", init_code).unwrap();

        // 执行脚本
        if let Err(e) = runtime.execute_script(script_path, script_code) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JS Error: {}", e),
            )
                .into_response();
        }
    }

    // 等待 JS 调用 op_send_response
    let js_res = match rx.await {
        Ok(res) => res,
        Err(_) => JsResponse {
            status: 500,
            headers: HashMap::new(),
            body: "JS failed to send response (did you forget to call Deno.core.ops.op_send_response?)".to_string(),
        },
    };

    let mut res_builder = axum::response::Response::builder().status(js_res.status);

    for (k, v) in js_res.headers {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(k), HeaderValue::try_from(v)) {
            res_builder = res_builder.header(name, value);
        }
    }

    res_builder.body(Body::from(js_res.body)).unwrap().into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/js/{*script_path}", any(handle_js_script));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
