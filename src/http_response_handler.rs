use axum::extract::State;
use std::sync::Arc;

use crate::event_aggregator::EventAggregator;
use crate::harnesses::HarnessResponse;

pub struct HTTPResponseHandler {
    event_aggregator: EventAggregator,
}

impl HTTPResponseHandler {
    pub fn new(event_aggregator: EventAggregator) -> Self {
        Self { event_aggregator }
    }

    pub async fn handle_post_response(State(handler): State<Arc<Self>>, body: String) -> String {
        handler.process_body(&body);
        "Ok".to_string()
    }

    fn process_body(&self, body: &str) {
        tracing::info!("Received the following data from Agent: {}", body);

        let harness_data: HarnessResponse =
            serde_json::from_str(body).expect("Failed to parse body as HarnessResponse");

        let timestamp = chrono::Utc::now().timestamp().to_string();

        self.event_aggregator.add_event_pair_from_raw_stream(
            harness_data.userInput,
            harness_data.agentOutput,
            timestamp,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_config::GlobalConfig;
    use crate::storage::traits::StorageBackend;
    use axum::routing::post;
    use tower::util::ServiceExt;

    fn test_handler() -> HTTPResponseHandler {
        let config = GlobalConfig {
            storage_backend: StorageBackend::Postgres,
            database_connection_string: "postgres://localhost:5432/test".into(),
        };
        HTTPResponseHandler::new(EventAggregator::new(config))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_process_body_with_valid_json() {
        let handler = test_handler();
        let body = r#"{"userInput":"hello","agentOutput":"world"}"#;
        handler.process_body(body);
    }

    #[tokio::test]
    #[should_panic(expected = "Failed to parse body as HarnessResponse")]
    async fn test_process_body_with_invalid_json() {
        let handler = test_handler();
        handler.process_body("not valid json");
    }

    #[tokio::test]
    #[should_panic(expected = "Failed to parse body as HarnessResponse")]
    async fn test_process_body_with_missing_fields() {
        let handler = test_handler();
        let body = r#"{"userInput":"hello"}"#;
        handler.process_body(body);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_post_response_returns_ok() {
        let handler = Arc::new(test_handler());
        let router = axum::Router::new()
            .route("/record", post(HTTPResponseHandler::handle_post_response))
            .with_state(handler);

        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/record")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"userInput":"hi","agentOutput":"bye"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);
    }
}
