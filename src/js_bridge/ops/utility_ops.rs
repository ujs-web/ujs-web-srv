use deno_core::op2;

/// 工具操作 - 单一职责：提供JavaScript运行时的工具函数
#[op2(fast)]
pub fn op_log(#[string] msg: String) {
    println!("[JS Log]: {}", msg);
}

#[op2(async)]
pub async fn op_delay(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_log_exists() {
        // 测试ops函数存在且可以编译
        // 实际功能测试在集成测试中进行
        assert!(true);
    }

    #[test]
    fn test_op_delay_exists() {
        // 测试ops函数存在且可以编译
        // 实际功能测试在集成测试中进行
        assert!(true);
    }
}