#[cfg(test)]
mod tests {
    use crate::js_bridge::executor::script_runner::ScriptRunner;
    use crate::js_bridge::models::JsRequest;
    use crate::js_bridge::{RuntimeConfig, ScriptExecutor};

    #[test]
    fn test_run_script_not_found() {
        let mut runtime =
            crate::js_bridge::executor::runtime_factory::RuntimeFactory::create_runtime();

        let result = ScriptRunner::run_script(&mut runtime, "./non_existent.js");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_script_not_found() {
        let pool = crate::db_bridge::establish_connection_pool();

        let request = JsRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: std::collections::HashMap::new(),
            body: String::new(),
        };

        let config = RuntimeConfig {
            script_path: "./non_existent.js".to_string(),
            request,
            db_pool: pool,
        };

        let response = ScriptExecutor::execute(config).await;
        assert_eq!(response.status, 404);
    }
}
