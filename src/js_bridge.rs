use axum::{
    body::Body,
    extract::{Path, Request},
    http::{HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use deno_ast::{MediaType, ParseParams};
use deno_core::{
    JsRuntime, ModuleLoadOptions, ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode,
    ModuleSpecifier, ModuleType, OpState, ResolutionKind, RuntimeOptions, extension, op2,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::oneshot;

struct TsModuleLoader;

impl ModuleLoader for TsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_error::JsErrorBox> {
        deno_core::resolve_import(specifier, referrer).map_err(deno_error::JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&deno_core::ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let module_specifier = module_specifier.clone();
        let fut = async move {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| deno_error::JsErrorBox::generic("Only file:// URLs are supported"))?;

            let media_type = MediaType::from_path(&path);
            let code = std::fs::read_to_string(&path).map_err(deno_error::JsErrorBox::from_err)?;

            let (transpiled_code, module_type) = match media_type {
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Jsx
                | MediaType::Tsx => {
                    let parsed = deno_ast::parse_module(ParseParams {
                        specifier: module_specifier.clone(),
                        text: Arc::from(code),
                        media_type,
                        capture_tokens: false,
                        scope_analysis: false,
                        maybe_syntax: None,
                    })
                    .map_err(deno_error::JsErrorBox::from_err)?;
                    let transpiled = parsed
                        .transpile(
                            &deno_ast::TranspileOptions {
                                ..Default::default()
                            },
                            &deno_ast::TranspileModuleOptions::default(),
                            &deno_ast::EmitOptions::default(),
                        )
                        .map_err(deno_error::JsErrorBox::from_err)?
                        .into_source();
                    (transpiled.text, ModuleType::JavaScript)
                }
                _ => (code, ModuleType::JavaScript),
            };

            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(transpiled_code.clone().into()),
                &module_specifier,
                None,
            ))
        };
        ModuleLoadResponse::Async(Box::pin(fut))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[op2(async)]
async fn op_delay(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

extension!(web_runtime, ops = [op_log, op_send_response, op_delay],);

#[op2(fast)]
fn op_log(#[string] msg: String) {
    println!("[JS Log]: {}", msg);
}

#[op2]
fn op_send_response(state: &mut OpState, #[serde] res: JsResponse) {
    let tx = state.take::<oneshot::Sender<JsResponse>>();
    let _ = tx.send(res);
}

pub async fn handle_js_script(Path(script_path): Path<String>, req: Request) -> impl IntoResponse {
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
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read body").into_response();
        }
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
    if !std::path::Path::new(&full_script_path).exists() {
        return (StatusCode::NOT_FOUND, "Script not found").into_response();
    }

    // 执行脚本并支持异步
    let (tx, rx) = oneshot::channel();

    std::thread::spawn(move || {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            module_loader: Some(Rc::new(TsModuleLoader)),
            ..Default::default()
        });

        runtime.op_state().borrow_mut().put(tx);

        // 将请求对象注入 JS 环境
        let req_json = serde_json::to_string(&js_req).unwrap();
        let init_code = format!("globalThis.request = {};", req_json);
        runtime.execute_script("<init>", init_code).unwrap();

        // 运行事件循环以支持 async/await 和 ES 模块
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_runtime.block_on(async {
            // 解析脚本路径为 ModuleSpecifier
            let cwd = std::env::current_dir().unwrap();
            let specifier = deno_core::resolve_path(&full_script_path, &cwd).unwrap();

            // 加载主模块
            let mod_id = runtime.load_main_es_module(&specifier).await.unwrap();

            // 执行模块
            let evaluation = runtime.mod_evaluate(mod_id);

            // 运行事件循环直到模块执行完成
            runtime.run_event_loop(Default::default()).await.unwrap();

            // 检查评估结果（如果模块有 top-level await，可能在这里完成）
            evaluation.await.unwrap();
        });
    });

    // 等待 JS 调用 op_send_response
    let js_res = rx.await.unwrap_or_else(|_| JsResponse {
        status: 500,
        headers: HashMap::new(),
        body: "JS failed to send response (did you forget to call Deno.core.ops.op_send_response?)"
            .to_string(),
    });

    let mut res_builder = axum::response::Response::builder().status(js_res.status);

    for (k, v) in js_res.headers {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(k), HeaderValue::try_from(v)) {
            res_builder = res_builder.header(name, value);
        }
    }

    res_builder
        .body(Body::from(js_res.body))
        .unwrap()
        .into_response()
}
