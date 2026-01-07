pub mod runtime_factory;
pub mod script_runner;

use crate::db_bridge::DbPool;
use crate::js_bridge::executor::runtime_factory::RuntimeFactory;
use crate::js_bridge::executor::script_runner::ScriptRunner;
use crate::js_bridge::models::{JsRequest, JsResponse};
use tokio::sync::oneshot;

/// 运行时配置
pub struct RuntimeConfig {
    pub script_path: String,
    pub request: JsRequest,
    pub db_pool: DbPool,
}

/// 脚本执行器 - 单一职责：协调整个脚本执行流程
pub struct ScriptExecutor;

impl ScriptExecutor {
    /// 执行脚本
    pub async fn execute(config: RuntimeConfig) -> JsResponse {
        let (tx, rx) = oneshot::channel();
        let script_path = config.script_path.clone();

        // 检查脚本文件是否存在
        if !std::path::Path::new(&script_path).exists() {
            return JsResponse::not_found("Script not found");
        }

        // 在独立线程中执行脚本
        rayon::spawn(move || {
            // 创建运行时
            let mut runtime = RuntimeFactory::create_runtime();

            // 配置运行时
            RuntimeFactory::configure_runtime(
                &mut runtime,
                config.request,
                config.db_pool,
                tx,
            );

            // 运行脚本
            if let Err(e) = ScriptRunner::run_script(&mut runtime, &script_path) {
                eprintln!("Script execution error: {}", e);
            }
        });

        // 等待响应
        rx.await.unwrap_or_else(|_| {
            JsResponse::internal_error("JS failed to send response (did you forget to call Deno.core.ops.op_send_response?)")
        })
    }
}
