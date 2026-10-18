use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;

use super::service::{ProjectCreatePayload, ProjectManagementService};

pub struct ProjectHTTPHandler {
    project_management_service: ProjectManagementService,
}

impl ProjectHTTPHandler {
    pub fn new(project_management_service: ProjectManagementService) -> Self {
        Self {
            project_management_service,
        }
    }

    pub fn router(self: Arc<Self>) -> axum::Router {
        axum::Router::new()
            .route(
                "/create_project",
                post(ProjectHTTPHandler::handle_http_create_project_lane),
            )
            .with_state(self)
    }

    pub async fn handle_http_create_project_lane(
        State(handler): State<Arc<Self>>,
        body: String,
    ) -> Result<String, String> {
        let payload: ProjectCreatePayload = serde_json::from_str(&body)
            .map_err(|_| "Failed to parse body for create project payload.".to_string())?;

        match handler
            .project_management_service
            .create_new_project_lane(payload.project_name, payload.timestamp)
            .await
        {
            Ok(id) => Ok(id.to_string()),
            Err(err) => {
                tracing::error!("Failed to create a project lane: {}", err);
                Err(format!(
                    "Encountered the following error while creating the project lane: {}",
                    err
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_log::agent_recorder::test_utils::MockStorageEngine;
    use std::sync::atomic::AtomicUsize;
    use tower::util::ServiceExt;

    async fn test_router() -> axum::Router {
        let engine = Box::new(MockStorageEngine {
            store_count: Arc::new(AtomicUsize::new(0)),
        });
        let service = ProjectManagementService::new(Arc::new(engine)).await;
        Arc::new(ProjectHTTPHandler::new(service)).router()
    }

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_handle_http_create_project_lane_with_valid_payload() {
        let router = test_router().await;
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/create_project")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"projectName":"proj","timestamp":"123"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(body_string(response).await, uuid::Uuid::nil().to_string());
    }

    #[tokio::test]
    async fn test_handle_http_create_project_lane_with_invalid_json() {
        let router = test_router().await;
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/create_project")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from("not valid json"))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(
            body_string(response).await,
            "Failed to parse body for create project payload."
        );
    }

    #[tokio::test]
    async fn test_handle_http_create_project_lane_with_missing_fields() {
        let router = test_router().await;
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/create_project")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"projectName":"proj"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(
            body_string(response).await,
            "Failed to parse body for create project payload."
        );
    }
}
