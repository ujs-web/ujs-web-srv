// Executor模块 - 已重构为模块化结构
// 为了向后兼容，保留此文件作为入口点
// 实际实现已迁移到 executor/ 模块

pub use crate::js_bridge::executor::{RuntimeConfig, ScriptExecutor};
