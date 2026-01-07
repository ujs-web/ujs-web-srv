use std::sync::OnceLock;
use crate::db_bridge::DbPool;

/// 获取全局测试连接池（单例模式）
pub fn get_test_pool() -> &'static DbPool {
    static POOL: OnceLock<DbPool> = OnceLock::new();
    POOL.get_or_init(|| {
        crate::db_bridge::establish_connection_pool()
    })
}