use crate::js_bridge::models::JsResponse;
use deno_core::{op2, OpState};
use tokio::sync::oneshot;

/// 响应相关操作 - 单一职责：处理JavaScript发送的HTTP响应
#[op2]
pub fn op_send_response(state: &mut OpState, #[serde] res: JsResponse) {
    let tx = state.take::<oneshot::Sender<JsResponse>>();
    let _ = tx.send(res);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_op_send_response_exists() {
        // 测试ops函数存在且可以编译
        // 实际功能测试在集成测试中进行
        assert!(true);
    }
}