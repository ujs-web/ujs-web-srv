pub mod batch_processor;
pub mod handler;
pub mod request_parser;
pub mod request_validator;
pub mod response_builder;

// 重新导出主要的处理器
pub use handler::handle_json_rpc;