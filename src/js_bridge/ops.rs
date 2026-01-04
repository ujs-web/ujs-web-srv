use crate::js_bridge::models::{JsRequest, JsResponse};
use deno_core::{extension, op2, OpState};
use std::collections::HashMap;
use tokio::sync::oneshot;

#[op2(async)]
pub async fn op_delay(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[op2]
#[string]
pub fn op_req_method(state: &mut OpState) -> String {
    let req = state.borrow::<JsRequest>();
    req.get_method()
}

#[op2]
#[string]
pub fn op_req_path(state: &mut OpState) -> String {
    let req = state.borrow::<JsRequest>();
    req.get_path()
}

#[op2]
#[serde]
pub fn op_req_headers(state: &mut OpState) -> HashMap<String, String> {
    let req = state.borrow::<JsRequest>();
    req.get_headers()
}

#[op2]
#[string]
pub fn op_req_body(state: &mut OpState) -> String {
    let req = state.borrow::<JsRequest>();
    req.get_body()
}

#[op2]
#[string]
pub fn op_req_get_header(state: &mut OpState, #[string] key: String) -> Option<String> {
    let req = state.borrow::<JsRequest>();
    req.get_header(&key)
}

#[op2(fast)]
pub fn op_log(#[string] msg: String) {
    println!("[JS Log]: {}", msg);
}

#[op2]
pub fn op_send_response(state: &mut OpState, #[serde] res: JsResponse) {
    let tx = state.take::<oneshot::Sender<JsResponse>>();
    let _ = tx.send(res);
}

extension!(
    web_runtime,
    ops = [
        op_log,
        op_send_response,
        op_delay,
        op_req_method,
        op_req_path,
        op_req_headers,
        op_req_body,
        op_req_get_header
    ],
);
