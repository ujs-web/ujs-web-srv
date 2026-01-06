use crate::js_bridge::models::JsRequest;
use deno_core::{op2, OpState};
use std::collections::HashMap;

/// 请求相关操作 - 单一职责：处理JavaScript对HTTP请求的访问
#[op2]
#[string]
pub fn op_req_method(state: &mut OpState, #[smi] rid: u32) -> String {
    let req = state
        .resource_table
        .get::<JsRequest>(rid)
        .expect("Failed to get JsRequest resource");
    req.get_method()
}

#[op2]
#[string]
pub fn op_req_path(state: &mut OpState, #[smi] rid: u32) -> String {
    let req = state
        .resource_table
        .get::<JsRequest>(rid)
        .expect("Failed to get JsRequest resource");
    req.get_path()
}

#[op2]
#[serde]
pub fn op_req_headers(state: &mut OpState, #[smi] rid: u32) -> HashMap<String, String> {
    let req = state
        .resource_table
        .get::<JsRequest>(rid)
        .expect("Failed to get JsRequest resource");
    req.get_headers()
}

#[op2]
#[string]
pub fn op_req_body(state: &mut OpState, #[smi] rid: u32) -> String {
    let req = state
        .resource_table
        .get::<JsRequest>(rid)
        .expect("Failed to get JsRequest resource");
    req.get_body()
}

#[op2]
#[string]
pub fn op_req_get_header(
    state: &mut OpState,
    #[smi] rid: u32,
    #[string] key: String,
) -> Option<String> {
    let req = state
        .resource_table
        .get::<JsRequest>(rid)
        .expect("Failed to get JsRequest resource");
    req.get_header(&key)
}

#[op2(fast)]
pub fn op_req_close(state: &mut OpState, #[smi] rid: u32) {
    if let Ok(resource) = state.resource_table.take_any(rid) {
        resource.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_req_ops_exist() {
        // 测试ops函数存在且可以编译
        // 实际功能测试在集成测试中进行
        assert!(true);
    }
}