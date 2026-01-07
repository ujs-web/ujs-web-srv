use crate::js_bridge::models::JsResponse;
use deno_core::JsRuntime;

/// 脚本运行器 - 单一职责：加载和执行JavaScript脚本
pub struct ScriptRunner;

impl ScriptRunner {
    /// 运行脚本
    pub fn run_script(runtime: &mut JsRuntime, script_path: &str) -> Result<(), String> {
        // 运行事件循环以支持 async/await 和 ES 模块
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

        tokio_runtime.block_on(async {
            Self::run_script_async(runtime, script_path).await
        })
    }

    /// 异步运行脚本
    async fn run_script_async(runtime: &mut JsRuntime, script_path: &str) -> Result<(), String> {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let specifier = deno_core::resolve_path(script_path, &cwd)
            .map_err(|e| format!("Failed to resolve script path: {}", e))?;

        // 加载主模块
        let mod_id = runtime
            .load_main_es_module(&specifier)
            .await
            .map_err(|e| format!("Failed to load module: {}", e))?;

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

        Ok(())
    }
}
