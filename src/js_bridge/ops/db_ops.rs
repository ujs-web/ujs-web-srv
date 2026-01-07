use crate::db_bridge::DbPool;
use deno_core::{OpState, op2};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::row::{Field, NamedRow, Row};
use serde_json::{Map, Value};

/// 动态行 - 用于存储数据库查询结果
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

/// 数据库相关操作 - 单一职责：处理JavaScript对数据库的访问
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

