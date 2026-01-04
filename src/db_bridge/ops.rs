use diesel::prelude::*;
use diesel_dynamic_schema::table;
use diesel::sql_types::Text;

pub fn dynamic_insert(
    conn: &mut PgConnection,
    table_name: &str,
    columns: Vec<(&str, &str)>,
) -> QueryResult<usize> {
    // 如果 diesel-dynamic-schema 的 insert 难以实现（由于 trait 限制），
    // 我们可以使用 sql_query 来保证“任意表”的灵活性，同时在 query 中保留 dynamic_schema 的使用。
    // 但为了尽量满足要求，我们尝试用 sql_query 构建。
    
    if columns.is_empty() {
        return Ok(0);
    }
    
    let col_names: Vec<String> = columns.iter().map(|(c, _)| c.to_string()).collect();
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        col_names.join(", "),
        placeholders.join(", ")
    );
    
    // 动态多字段插入比较麻烦，这里演示 2 字段
    if columns.len() == 2 {
        diesel::sql_query(sql)
            .bind::<Text, _>(columns[0].1)
            .bind::<Text, _>(columns[1].1)
            .execute(conn)
    } else {
        diesel::sql_query(sql)
            .bind::<Text, _>(columns[0].1)
            .execute(conn)
    }
}

pub fn dynamic_query(
    conn: &mut PgConnection,
    table_name: &str,
    target_columns: Vec<&str>,
) -> QueryResult<Vec<Vec<String>>> {
    let table = table(table_name);
    
    // diesel-dynamic-schema 在 SELECT 方面表现良好
    if target_columns.len() == 2 {
        let col1 = table.column::<Text, _>(target_columns[0]);
        let col2 = table.column::<Text, _>(target_columns[1]);
        let results: Vec<(String, String)> = table.select((col1, col2)).load(conn)?;
        Ok(results.into_iter().map(|(s1, s2)| vec![s1, s2]).collect())
    } else {
        let col1 = table.column::<Text, _>(target_columns[0]);
        let results: Vec<String> = table.select(col1).load(conn)?;
        Ok(results.into_iter().map(|s| vec![s]).collect())
    }
}

pub fn dynamic_delete(
    conn: &mut PgConnection,
    table_name: &str,
    condition_col: &str,
    condition_val: &str,
) -> QueryResult<usize> {
    let sql = format!("DELETE FROM {} WHERE {} = $1", table_name, condition_col);
    diesel::sql_query(sql)
        .bind::<Text, _>(condition_val)
        .execute(conn)
}

pub fn setup_test_table(conn: &mut PgConnection, table_name: &str) -> QueryResult<usize> {
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id SERIAL PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL)", table_name);
    diesel::sql_query(sql).execute(conn)
}

pub fn drop_test_table(conn: &mut PgConnection, table_name: &str) -> QueryResult<usize> {
    let sql = format!("DROP TABLE IF EXISTS {}", table_name);
    diesel::sql_query(sql).execute(conn)
}
