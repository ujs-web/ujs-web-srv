use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://ever@localhost/postgres".to_string());
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
pub mod ops;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_bridge::ops::*;

    #[test]
    fn test_db_operations() {
        let pool = establish_connection_pool();
        let mut conn = pool.get().expect("Failed to get connection from pool");
        let table_name = "dynamic_users";

        // 0. Cleanup first (in case previous test failed)
        let _ = drop_test_table(&mut conn, table_name);

        // 1. Setup
        let _ = setup_test_table(&mut conn, table_name).expect("Failed to setup table");

        // 2. Insert
        let columns = vec![
            ("name", "dynamic_user"),
            ("email", "dynamic@example.com"),
        ];
        dynamic_insert(&mut conn, table_name, columns).expect("Failed to create user");

        // 3. Query
        let query_cols = vec!["name", "email"];
        let results = dynamic_query(&mut conn, table_name, query_cols).expect("Failed to query user");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0][0], "dynamic_user");
        assert_eq!(results[0][1], "dynamic@example.com");

        // 4. Delete
        dynamic_delete(&mut conn, table_name, "name", "dynamic_user").expect("Failed to delete user");
        let query_cols_after = vec!["name"];
        let results_after = dynamic_query(&mut conn, table_name, query_cols_after).expect("Failed to query user count");
        assert_eq!(results_after.len(), 0);

        // 5. Cleanup
        let _ = drop_test_table(&mut conn, table_name).expect("Failed to drop table");
    }
}
