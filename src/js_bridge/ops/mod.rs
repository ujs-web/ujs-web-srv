pub mod db_ops;
pub mod request_ops;
pub mod response_ops;
pub mod utility_ops;

// 创建扩展，包含所有操作
deno_core::extension!(
    web_runtime,
    ops = [
        // 工具操作
        utility_ops::op_log,
        utility_ops::op_delay,
        // 响应操作
        response_ops::op_send_response,
        // 请求操作
        request_ops::op_req_close,
        request_ops::op_req_method,
        request_ops::op_req_path,
        request_ops::op_req_headers,
        request_ops::op_req_body,
        request_ops::op_req_get_header,
        // 数据库操作
        db_ops::op_sql_execute,
        db_ops::op_sql_query
    ],
    esm_entry_point = "ext:web_runtime/init.js",
    esm = [ dir "src/js_bridge", "init.js" ],
);