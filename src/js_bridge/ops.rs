use crate::db_bridge::DbPool;
use crate::js_bridge::models::{JsRequest, JsResponse};
use deno_core::{OpState, extension, op2};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::row::{Field, NamedRow, Row};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tokio::sync::oneshot;

#[derive(serde::Serialize)]
pub struct DynamicRow(pub Map<String, Value>);

impl diesel::deserialize::QueryableByName<Pg> for DynamicRow {
    fn build<'a>(
        row: &impl NamedRow<'a, Pg>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        use diesel::deserialize::FromSql;
        let mut map = Map::new();
        let column_count = row.field_count();

        for i in 0..column_count {
            let field = Row::get(row, i).ok_or_else(|| "Failed to get field")?;
            let name = field
                .field_name()
                .ok_or_else(|| "Failed to get column name")?;

            let value = if field.is_null() {
                Value::Null
            } else {
                let raw_value = field.value().unwrap();
                // 尝试多种类型，通过顺序保证优先匹配
                // 在 PostgreSQL 二进制协议下，整数的长度是固定的
                if let Ok(v) = FromSql::<diesel::sql_types::Integer, Pg>::from_sql(raw_value) {
                    let v: i32 = v;
                    Value::Number(serde_json::Number::from(v))
                } else if let Ok(v) = FromSql::<diesel::sql_types::Text, Pg>::from_sql(raw_value) {
                    let v: String = v;
                    Value::String(v)
                } else if let Ok(v) = FromSql::<diesel::sql_types::BigInt, Pg>::from_sql(raw_value)
                {
                    let v: i64 = v;
                    Value::Number(serde_json::Number::from(v))
                } else if let Ok(v) = FromSql::<diesel::sql_types::Double, Pg>::from_sql(raw_value)
                {
                    let v: f64 = v;
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                } else if let Ok(v) = FromSql::<diesel::sql_types::Bool, Pg>::from_sql(raw_value) {
                    let v: bool = v;
                    Value::Bool(v)
                } else {
                    String::from_utf8(raw_value.as_bytes().to_vec())
                        .map(Value::String)
                        .unwrap_or_else(|_| {
                            Value::String(format!("Binary: {} bytes", raw_value.as_bytes().len()))
                        })
                }
            };

            map.insert(name.to_string(), value);
        }
        Ok(DynamicRow(map))
    }
}

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

    let rows = diesel::sql_query(sql)
        .load::<DynamicRow>(&mut conn)
        .unwrap_or_default();

    serde_json::to_value(rows).unwrap()
}

#[op2(async)]
pub async fn op_delay(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}
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
        op_req_close,
        op_req_method,
        op_req_path,
        op_req_headers,
        op_req_body,
        op_req_get_header,
        op_sql_execute,
        op_sql_query
    ],
    esm_entry_point = "ext:web_runtime/init.js", // 路径必须是 ext:<扩展名>/<文件名>，运行时才能识别
    esm = [ dir "src/js_bridge", "init.js" ], // 打包
);
