use crate::db_bridge::DbPool;
use crate::js_bridge::loader::TsModuleLoader;
use crate::js_bridge::models::{JsRequest, JsResponse};
use crate::js_bridge::ops::web_runtime;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use std::rc::Rc;
use tokio::sync::oneshot;

/// 运行时工厂 - 单一职责：创建和配置JavaScript运行时
pub struct RuntimeFactory;

impl RuntimeFactory {
    /// 创建新的JavaScript运行时
    pub fn create_runtime() -> JsRuntime {
        JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            module_loader: Some(Rc::new(TsModuleLoader)),
            ..Default::default()
        })
    }

    /// 配置运行时状态
    pub fn configure_runtime(
        runtime: &mut JsRuntime,
        request: JsRequest,
        db_pool: DbPool,
        tx: oneshot::Sender<JsResponse>,
    ) -> u32 {
        // 设置响应通道
        runtime.op_state().borrow_mut().put(tx);

        // 添加请求资源
        let rid = runtime.op_state().borrow_mut().resource_table.add(request);

        // 注入RID到全局作用域
        let init_code = format!(
            r#"
    globalThis.__JS_REQUEST_RID__ = {};
            "#,
            rid
        );
        runtime
            .execute_script("<init_rid>", init_code)
            .expect("Failed to inject RID");

        // 设置数据库连接池
        runtime.op_state().borrow_mut().put(db_pool);

        rid
    }
}
