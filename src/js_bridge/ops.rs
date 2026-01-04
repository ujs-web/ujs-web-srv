use crate::db_bridge::DbPool;
use crate::js_bridge::models::{JsRequest, JsResponse};
use diesel::prelude::*;
use deno_core::{extension, op2, OpState};
use std::collections::HashMap;
use tokio::sync::oneshot;

#[op2(fast)]
pub fn op_sql_execute(state: &mut OpState, #[string] sql: String) -> u32 {
    let pool = state.borrow::<DbPool>();
    let mut conn = pool.get().expect("Failed to get connection from pool");
    diesel::sql_query(sql)
        .execute(&mut conn)
        .expect("SQL execution failed") as u32
}

#[op2]
#[serde]
pub fn op_sql_query(state: &mut OpState, #[string] sql: String) -> serde_json::Value {
    let pool = state.borrow::<DbPool>();
    let mut conn = pool.get().expect("Failed to get connection from pool");
    
    // 为了支持动态查询结果，我们使用一个局部定义的结构体，
    // 虽然它受限于列名，但在 sql_query 中可以通过 AS 别名来适配。
    // 或者我们直接返回 JSON 数组（如果能从驱动获取）。
    
    #[derive(QueryableByName, serde::Serialize)]
    struct RawResult {
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub res1: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub res2: String,
    }

    // 尝试执行查询并返回前两列（如果存在）
    // 注意：这要求 SQL 语句必须 select 两个文本字段，并分别命名为 res1, res2
    // 例如: SELECT name as res1, email as res2 FROM users
    let rows = diesel::sql_query(sql)
        .load::<RawResult>(&mut conn)
        .unwrap_or_default();
    
    serde_json::to_value(rows).unwrap()
}

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
        op_req_get_header,
        op_sql_execute,
        op_sql_query
    ],
);
