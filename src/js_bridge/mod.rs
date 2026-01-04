pub mod executor;
pub mod handler;
pub mod loader;
pub mod models;
pub mod ops;

#[allow(unused_imports)]
pub use executor::{RuntimeConfig, ScriptExecutor};
pub use handler::handle_js_script;
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
        let req = Request::builder()
            .method("POST")
            .uri("/js/test_request.js")
            .header("X-Test", "value")
            .body(Body::from("hello world"))
            .unwrap();

        let response = handle_js_script(Path("test_request.js".to_string()), req).await;
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
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
        let req = Request::builder()
            .method("GET")
            .uri("/js/async_test.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(Path("async_test.js".to_string()), req).await;
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
        let req = Request::builder()
            .method("GET")
            .uri("/js/non_existent.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(Path("non_existent.js".to_string()), req).await;
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

        let response = handle_js_script(Path("test_ts_transpile.ts".to_string()), req).await;
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
        let bad_code = "this is not javascript";
        std::fs::write("./scripts/bad_syntax.js", bad_code).unwrap();

        let req = Request::builder()
            .method("GET")
            .uri("/js/bad_syntax.js")
            .body(Body::empty())
            .unwrap();

        let response = handle_js_script(Path("bad_syntax.js".to_string()), req).await;
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
        runtime.op_state().borrow_mut().put(js_req);

        // 使用脚本验证
        let code = r#"
            if (Deno.core.ops.op_req_method() !== "PUT") throw new Error("Method mismatch");
            if (Deno.core.ops.op_req_path() !== "/op-test") throw new Error("Path mismatch");
            "ok";
        "#;
        let result = runtime.execute_script("<test>", code);
        // 如果没有抛出错误，说明测试通过
        assert!(result.is_ok());
    }
}
