use crate::db_bridge::DbPool;
use crate::js_bridge::loader::TsModuleLoader;
use crate::js_bridge::models::{JsRequest, JsResponse};
use crate::js_bridge::ops::web_runtime;
use deno_core::{JsRuntime, RuntimeOptions};
use std::rc::Rc;
use tokio::sync::oneshot;

pub struct RuntimeConfig {
    pub script_path: String,
    pub request: JsRequest,
    pub db_pool: DbPool,
}

pub struct ScriptExecutor;

impl ScriptExecutor {
    pub async fn execute(config: RuntimeConfig) -> JsResponse {
        let (tx, rx) = oneshot::channel();
        let script_path = config.script_path.clone();

        if !std::path::Path::new(&script_path).exists() {
            return JsResponse::not_found("Script not found");
        }

        rayon::spawn(move || {
            let mut runtime = JsRuntime::new(RuntimeOptions {
                extensions: vec![web_runtime::init()],
                module_loader: Some(Rc::new(TsModuleLoader)),
                ..Default::default()
            });

            runtime.op_state().borrow_mut().put(tx);
            runtime.op_state().borrow_mut().put(config.request);
            runtime.op_state().borrow_mut().put(config.db_pool);

            // 将请求对象注入 JS 环境，使用方法访问
            let init_code = r#"
                globalThis.request = {
                    method: () => Deno.core.ops.op_req_method(),
                    path: () => Deno.core.ops.op_req_path(),
                    headers: () => Deno.core.ops.op_req_headers(),
                    body: () => Deno.core.ops.op_req_body(),
                    header: (key) => Deno.core.ops.op_req_get_header(key),
                };
                globalThis.db = {
                    execute: (sql) => Deno.core.ops.op_sql_execute(sql),
                    query: (sql) => Deno.core.ops.op_sql_query(sql),
                };
            "#;
            if let Err(e) = runtime.execute_script("<init>", init_code) {
                eprintln!("Failed to execute init script: {}", e);
                return;
            }

            // 运行事件循环以支持 async/await 和 ES 模块
            let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            tokio_runtime.block_on(async {
                let cwd = std::env::current_dir().unwrap();
                let specifier = match deno_core::resolve_path(&script_path, &cwd) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to resolve script path: {}", e);
                        return;
                    }
                };

                // 加载主模块
                let mod_id = match runtime.load_main_es_module(&specifier).await {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("Failed to load module: {}", e);
                        return;
                    }
                };

                // 执行模块
                let evaluation = runtime.mod_evaluate(mod_id);

                // 运行事件循环直到模块执行完成
                if let Err(e) = runtime.run_event_loop(Default::default()).await {
                    eprintln!("Event loop error: {}", e);
                }

                // 检查评估结果
                if let Err(e) = evaluation.await {
                    eprintln!("Module evaluation error: {}", e);
                }
            });
        });

        rx.await.unwrap_or_else(|_| {
            JsResponse::internal_error("JS failed to send response (did you forget to call Deno.core.ops.op_send_response?)")
        })
    }
}
