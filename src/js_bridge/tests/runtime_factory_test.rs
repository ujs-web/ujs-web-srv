#[cfg(test)]
mod tests {
    use crate::js_bridge::models::JsRequest;
    use std::collections::HashMap;
    use tokio::sync::oneshot;
    use crate::js_bridge::executor::runtime_factory::RuntimeFactory;

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
