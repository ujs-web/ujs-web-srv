use crate::db_bridge::DbPool;
use crate::js_bridge::loader::TsModuleLoader;
use crate::js_bridge::models::JsRequest;
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
        tx: oneshot::Sender<crate::js_bridge::models::JsResponse>,
    ) -> u32 {
        // 设置响应通道
        runtime.op_state().borrow_mut().put(tx);

        // 添加请求资源
        let rid = runtime
            .op_state()
            .borrow_mut()
            .resource_table
            .add(request);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_bridge::models::JsRequest;
    use std::collections::HashMap;
    use tokio::sync::oneshot;

    #[test]
    fn test_create_runtime() {
        let runtime = RuntimeFactory::create_runtime();
        // 运行时创建成功即可
        assert!(runtime.op_state().borrow().resource_table.is_empty());
    }

    #[test]
    fn test_configure_runtime() {
        let mut runtime = RuntimeFactory::create_runtime();
        let (tx, _rx) = oneshot::channel();

        let request = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            body: "test".to_string(),
        };

        let pool = crate::db_bridge::establish_connection_pool();

        let rid = RuntimeFactory::configure_runtime(&mut runtime, request, pool, tx);

        // 验证RID已设置
        let code = format!(
            r#"
            if (globalThis.__JS_REQUEST_RID__ !== {}) throw new Error("RID not set");
            "ok";
            "#,
            rid
        );
        let result = runtime.execute_script("<test>", code);
        assert!(result.is_ok());
    }
}