use std::sync::Arc;

use crate::storage::traits::StorageEngine;

pub struct ProjectManagementService {
    storage: Arc<Box<dyn StorageEngine>>,
}

impl ProjectManagementService {
    pub async fn new(storage: Arc<Box<dyn StorageEngine>>) -> Self {
        ProjectManagementService { storage }
    }
}
