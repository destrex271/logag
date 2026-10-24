use std::sync::Arc;

use rmcp::{handler::server::wrapper::Parameters, tool_router};
use serde::{Deserialize, Serialize};

use crate::storage::traits::{StorageEngine, StorageEngineErrors};

#[derive(Clone)]
pub struct ProjectManagementService {
    storage: Arc<Box<dyn StorageEngine>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreatePayload {
    pub project_name: String,
    pub timestamp: String,
}

impl ProjectManagementService {
    pub async fn new(storage: Arc<Box<dyn StorageEngine>>) -> Self {
        ProjectManagementService { storage }
    }

    pub async fn create_new_project_lane(
        &self,
        project_name: String,
        timestamp: String,
    ) -> Result<uuid::Uuid, StorageEngineErrors> {
        match self
            .storage
            .store_project_lane(project_name, timestamp)
            .await
        {
            Ok(uuid) => Ok(uuid),
            Err(err) => panic!("Failed to create project lane: {:?}", err),
        }
    }
}

#[tool_router(server_handler)]
impl ProjectManagementService{
    pub fn create_project_by_agent(
        &self,
        Parameters(
            ProjectCreatePayload { project_name, timestamp }
        ): Parameters<ProjectCreatePayload>,
    ) -> String {
        let value = tokio::runtime::Handle::current().block_on(async {
            return self.create_new_project_lane(project_name, timestamp).await.unwrap();
        });
        return value.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_log::agent_recorder::test_utils::MockStorageEngine;
    use std::sync::atomic::AtomicUsize;

    async fn test_service() -> ProjectManagementService {
        let engine = Box::new(MockStorageEngine {
            store_count: Arc::new(AtomicUsize::new(0)),
        });
        ProjectManagementService::new(Arc::new(engine)).await
    }

    #[tokio::test]
    async fn test_create_new_project_lane_success() {
        let service = test_service().await;
        let id = service
            .create_new_project_lane("proj".into(), "123".into())
            .await
            .unwrap();
        assert_eq!(id, uuid::Uuid::nil());
    }

    #[test]
    fn test_project_create_payload_deserializes_camel_case() {
        let json = r#"{"projectName":"proj","timestamp":"123"}"#;
        let payload: ProjectCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.project_name, "proj");
        assert_eq!(payload.timestamp, "123");
    }

    #[test]
    fn test_project_create_payload_serializes_camel_case() {
        let payload = ProjectCreatePayload {
            project_name: "proj".into(),
            timestamp: "123".into(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("projectName"));
        assert!(!json.contains("project_name"));
    }

    #[test]
    fn test_project_create_payload_missing_fields_fails() {
        let json = r#"{"projectName":"proj"}"#;
        let result: Result<ProjectCreatePayload, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
