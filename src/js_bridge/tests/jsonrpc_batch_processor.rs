#[cfg(test)]
mod tests {
    use crate::js_bridge::jsonrpc::batch_processor::BatchProcessor;
    use crate::js_bridge::models::JsonRpcRequest;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_process_batch() {
        let pool = crate::db_bridge::establish_connection_pool();
        let headers = HashMap::new();

        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "add".to_string(),
                params: Some(json!({"a": 1, "b": 2})),
                id: Some(json!(1)),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "multiply".to_string(),
                params: Some(json!({"a": 3, "b": 4})),
                id: Some(json!(2)),
            },
        ];

        let responses = BatchProcessor::process_batch(requests, pool, headers).await;
        assert_eq!(responses.len(), 2);
        assert!(responses[0].result.is_some());
        assert!(responses[1].result.is_some());
    }

    #[tokio::test]
    async fn test_process_batch_with_errors() {
        let pool = crate::db_bridge::establish_connection_pool();
        let headers = HashMap::new();

        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "add".to_string(),
                params: Some(json!({"a": 1, "b": 2})),
                id: Some(json!(1)),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "nonexistent".to_string(),
                params: Some(json!({})),
                id: Some(json!(2)),
            },
        ];

        let responses = BatchProcessor::process_batch(requests, pool, headers).await;
        assert_eq!(responses.len(), 2);
        assert!(responses[0].result.is_some());
        assert!(responses[1].error.is_some());
    }
}
