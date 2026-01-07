use deno_core::{JsRuntime, RuntimeOptions};
use super::super::models::JsRequest;
use super::super::ops::web_runtime;

#[tokio::test]
async fn test_op_req_ops_directly() {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![web_runtime::init()],
        ..Default::default()
    });
    let js_req = JsRequest {
        method: "PUT".to_string(),
        path: "/op-test".to_string(),
        headers: std::collections::HashMap::new(),
        body: "op-body".to_string(),
    };

    let rid = runtime.op_state().borrow_mut().resource_table.add(js_req);
    runtime.execute_script("<test_init>", format!("globalThis.__JS_REQUEST_RID__ = {};", rid)).unwrap();

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