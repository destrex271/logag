use std::sync::Arc;

use crate::storage::traits::{StorageEngine, StorageEngineErrors};

pub struct ProjectManagementService {
    storage: Arc<Box<dyn StorageEngine>>,
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
