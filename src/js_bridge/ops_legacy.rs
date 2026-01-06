// Ops模块 - 已重构为模块化结构
// 为了向后兼容，保留此文件作为入口点
// 实际实现已迁移到 ops/ 模块

pub use crate::js_bridge::ops::web_runtime;

pub use crate::js_bridge::ops::web_runtime as ops;

// 重新导出所有操作
pub use crate::js_bridge::ops::{
    op_log, op_send_response, op_delay, op_req_close, op_req_method, op_req_path,
    op_req_headers, op_req_body, op_req_get_header, op_sql_execute, op_sql_query, DynamicRow
};

#[cfg(test)]
mod tests {
    use super::*;
    use deno_core::{JsRuntime, RuntimeOptions};
    use std::collections::HashMap;
    use crate::js_bridge::models::{JsRequest, JsResponse};
    use tokio::sync::oneshot;

    #[test]
    fn test_ops_extension() {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            ..Default::default()
        });

        // 测试 op_log
        let code = r#"
            Deno.core.ops.op_log("test from ops");
            "ok";
        "#;
        let result = runtime.execute_script("<test_log>", code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_op_req_ops_integration() {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            ..Default::default()
        });

        let js_req = JsRequest {
            method: "PUT".to_string(),
            path: "/op-test".to_string(),
            headers: HashMap::new(),
            body: "op-body".to_string(),
        };

        let rid = runtime.op_state().borrow_mut().resource_table.add(js_req);
        runtime.execute_script("<test_init>", format!("globalThis.__JS_REQUEST_RID__ = {};", rid)).unwrap();

        // 使用脚本验证
        let code = format!(
            r#"
            const rid = globalThis.__JS_REQUEST_RID__;
            if (rid !== {}) throw new Error("RID mismatch");
            if (Deno.core.ops.op_req_method(rid) !== "PUT") throw new Error("Method mismatch");
            if (Deno.core.ops.op_req_path(rid) !== "/op-test") throw new Error("Path mismatch");
            Deno.core.ops.op_req_close(rid);
            "ok";
            "#,
            rid
        );
        let result = runtime.execute_script("<test>", code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_op_send_response_integration() {
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            ..Default::default()
        });
        let (tx, rx) = oneshot::channel();

        runtime.op_state().borrow_mut().put(tx);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let js_res = JsResponse {
            status: 200,
            headers,
            body: r#"{"test":"ok"}"#.to_string(),
        };

        op_send_response(&mut runtime.op_state().borrow_mut(), js_res);

        let received = rx.blocking_recv().unwrap();
        assert_eq!(received.status, 200);
        assert_eq!(received.body, r#"{"test":"ok"}"#);
    }
}