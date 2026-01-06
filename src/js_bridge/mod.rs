pub mod executor;
pub mod handler;
pub mod loader;
pub mod models;
pub mod ops;
pub mod jsonrpc_handler;

#[allow(unused_imports)]
pub use executor::{RuntimeConfig, ScriptExecutor};
pub use handler::handle_js_script;
pub use jsonrpc_handler::handle_json_rpc;
#[allow(unused_imports)]
pub use loader::TsModuleLoader;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::Path;
    use axum::http::{Request, StatusCode};
    use axum::response::IntoResponse;
    use deno_core::{JsRuntime, RuntimeOptions};
    use std::collections::HashMap;
    use crate::js_bridge::models::{JsRequest, JsResponse};
    use crate::js_bridge::ops::web_runtime;

    #[tokio::test]
    async fn test_js_request_methods() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        let pool = establish_connection_pool();
        let req = Request::builder()
            .method("POST")
            .uri("/js/test_request.js")
            .header("X-Test", "value")
            .body(Body::from("hello world"))
            .unwrap();

        let response = handle_js_script(State(pool), Path("test_request.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["method"], "POST");
        assert_eq!(json["path"], "/js/test_request.js");
        assert_eq!(json["headers"]["x-test"], "value");
        assert_eq!(json["body"], "hello world");
        assert_eq!(json["x_test"], "value");
        assert!(json["non_existent"].is_null());
    }

    #[tokio::test]
    async fn test_js_async_with_threadpool() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        let pool = establish_connection_pool();
        let req = Request::builder()
            .method("GET")
            .uri("/js/async_test.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("async_test.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["message"], "Hello from async JS");
    }

    #[tokio::test]
    async fn test_script_not_found() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        let pool = establish_connection_pool();
        let req = Request::builder()
            .method("GET")
            .uri("/js/non_existent.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("non_existent.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_request_getters() {
        let mut headers = HashMap::new();
        headers.insert("X-Key".to_string(), "Value".to_string());
        let req = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: headers.clone(),
            body: "body".to_string(),
        };

        assert_eq!(req.get_method(), "GET");
        assert_eq!(req.get_path(), "/test");
        assert_eq!(req.get_headers(), headers);
        assert_eq!(req.get_body(), "body");
        assert_eq!(req.get_header("X-Key"), Some("Value".to_string()));
        assert_eq!(req.get_header("Non-Existent"), None);
    }

    #[tokio::test]
    async fn test_ts_transpilation() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        let pool = establish_connection_pool();
        // 创建一个临时 TS 文件进行测试
        let ts_code = r#"
            interface User { name: string; }
            const user: User = { name: "TS User" };
            const res = {
                status: 200,
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ message: `Hello from ${user.name}` })
            };
            Deno.core.ops.op_send_response(res);
        "#;
        std::fs::create_dir_all("./scripts").unwrap();
        std::fs::write("./scripts/test_ts_transpile.ts", ts_code).unwrap();

        let req = Request::builder()
            .method("GET")
            .uri("/js/test_ts_transpile.ts")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("test_ts_transpile.ts".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&body_bytes)).unwrap();
        assert_eq!(json["message"], "Hello from TS User");

        // 清理
        let _ = std::fs::remove_file("./scripts/test_ts_transpile.ts");
    }

    #[tokio::test]
    async fn test_js_response_into_response() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "Value".to_string());
        let js_res = JsResponse {
            status: 201,
            headers,
            body: "created".to_string(),
        };

        let res = js_res.into_response();
        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(res.headers().get("X-Custom").unwrap(), "Value");
    }

    #[tokio::test]
    async fn test_js_syntax_error_handling() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        let pool = establish_connection_pool();
        let bad_code = "this is not javascript";
        std::fs::write("./scripts/bad_syntax.js", bad_code).unwrap();

        let req = Request::builder()
            .method("GET")
            .uri("/js/bad_syntax.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("bad_syntax.js".to_string()), req).await;
        let response = response.into_response();

        // 目前的代码在 JS 错误时会打印日志并可能超时，返回 500
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let _ = std::fs::remove_file("./scripts/bad_syntax.js");
    }

    #[tokio::test]
    async fn test_op_req_ops_directly() {
        // 测试 Deno ops 是否正确借用状态
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![web_runtime::init()],
            ..Default::default()
        });
        let js_req = JsRequest {
            method: "PUT".to_string(),
            path: "/op-test".to_string(),
            headers: HashMap::new(),
            body: "op-body".to_string(),
        };

        let rid = runtime.op_state().borrow_mut().resource_table.add(js_req);
        runtime.execute_script("<test_init>", format!("globalThis.__JS_REQUEST_RID__ = {};", rid)).unwrap();

        // 使用脚本验证
        let code = format!(
            r#"
            const rid = globalThis.__JS_REQUEST_RID__;
            if (rid !== {}) throw new Error("RID mismatch");
            if (Deno.core.ops.op_req_method(rid) !== "PUT") throw new Error("Method mismatch");
            if (Deno.core.ops.op_req_path(rid) !== "/op-test") throw new Error("Path mismatch");
            Deno.core.ops.op_req_close(rid);
            "ok";
            "#,
            rid
        );
        let result = runtime.execute_script("<test>", code);
        // 如果没有抛出错误，说明测试通过
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_js_sql_operations() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;

        let pool = establish_connection_pool();
        let req = Request::builder()
            .method("GET")
            .uri("/js/db_test.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("db_test.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["setup"], "ok");
        assert_eq!(json["inserted"], 1);
        // 注意：现在返回的是原始列名，不再是 res1
        assert_eq!(json["queried"][0]["name"], "js_user");
        assert_eq!(json["queried"][0]["res2"], "js@example.com");
        assert_eq!(json["updated_email"], "updated@example.com");
        assert_eq!(json["deleted"], "ok");
    }

    #[tokio::test]
    async fn test_js_sql_dynamic_row() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;

        let pool = establish_connection_pool();
        let req = Request::builder()
            .method("GET")
            .uri("/js/db_dynamic_test.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(State(pool), Path("db_dynamic_test.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["count"], 2);
        assert_eq!(json["first_row"]["name"], "Alice");
        assert_eq!(json["first_row"]["age"], 30); // 确保是数字类型
        assert_eq!(json["first_row"]["metadata"], "developer");
        assert_eq!(json["subset"][1]["name"], "Bob");
        assert!(json["subset"][1]["age"].is_null()); // subset 不包含 age
    }

    #[tokio::test]
    async fn test_json_rpc_add() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::{JsonRpcRequest, JsonRpcResponse};

        let pool = establish_connection_pool();
        let json_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "add".to_string(),
            params: Some(serde_json::json!({"a": 5, "b": 3})),
            id: Some(serde_json::json!(1)),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_req).unwrap()))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.error.is_none());
        assert_eq!(json_res.id, Some(serde_json::json!(1)));
        assert_eq!(json_res.result.unwrap()["result"], 8);
    }

    #[tokio::test]
    async fn test_json_rpc_multiply() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::{JsonRpcRequest, JsonRpcResponse};

        let pool = establish_connection_pool();
        let json_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "multiply".to_string(),
            params: Some(serde_json::json!({"a": 4, "b": 7})),
            id: Some(serde_json::json!(2)),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_req).unwrap()))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.error.is_none());
        assert_eq!(json_res.id, Some(serde_json::json!(2)));
        assert_eq!(json_res.result.unwrap()["result"], 28);
    }

    #[tokio::test]
    async fn test_json_rpc_greet() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::{JsonRpcRequest, JsonRpcResponse};

        let pool = establish_connection_pool();
        let json_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "greet".to_string(),
            params: Some(serde_json::json!({"name": "World"})),
            id: Some(serde_json::json!(3)),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_req).unwrap()))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.error.is_none());
        assert_eq!(json_res.id, Some(serde_json::json!(3)));
        assert_eq!(json_res.result.unwrap()["greeting"], "Hello, World!");
    }

    #[tokio::test]
    async fn test_json_rpc_method_not_found() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::{JsonRpcRequest, JsonRpcResponse};

        let pool = establish_connection_pool();
        let json_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "non_existent_method".to_string(),
            params: None,
            id: Some(serde_json::json!(4)),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_req).unwrap()))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.result.is_none());
        assert!(json_res.error.is_some());
        assert_eq!(json_res.id, Some(serde_json::json!(4)));
        assert_eq!(json_res.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_json_rpc_invalid_json() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::JsonRpcResponse;

        let pool = establish_connection_pool();

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "application/json")
            .body(Body::from("{invalid json}"))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.result.is_none());
        assert!(json_res.error.is_some());
        assert_eq!(json_res.error.unwrap().code, -32700);
    }

    #[tokio::test]
    async fn test_json_rpc_invalid_request() {
        use crate::db_bridge::establish_connection_pool;
        use axum::extract::State;
        use crate::js_bridge::models::JsonRpcResponse;

        let pool = establish_connection_pool();

        let req = Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("Content-Type", "text/plain")
            .body(Body::from("{}"))
            .unwrap();

        let response = handle_json_rpc(State(pool), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Response body: {}", body_str);

        let json_res: JsonRpcResponse = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json_res.jsonrpc, "2.0");
        assert!(json_res.result.is_none());
        assert!(json_res.error.is_some());
        assert_eq!(json_res.error.unwrap().code, -32600);
    }
}
