use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    global_config::GlobalConfig,
    shared_log::{log::LogContent, traits::Event, user_embedding_model::SlimUserEmbeddingInput},
    storage::postgres::PostgresStorage,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Postgres,
}

#[derive(Debug)]
pub struct UnknownStorageBackendError;

impl std::fmt::Display for UnknownStorageBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown storage backend")
    }
}

impl FromStr for StorageBackend {
    type Err = UnknownStorageBackendError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(StorageBackend::Postgres),
            _ => Err(UnknownStorageBackendError),
        }
    }
}

pub struct StorageBackendProvider {}

impl StorageBackendProvider {
    pub async fn get_storage_backend(config: GlobalConfig) -> Box<dyn StorageEngine> {
        match config.storage_backend {
            StorageBackend::Postgres => Box::new(PostgresStorage::load_storage(config).await),
        }
    }
}

#[derive(Debug)]
pub enum StorageEngineErrors {
    InvalidTimestamp(isize),
    DatabaseError(Box<dyn Error>),
    NoDataForField(String),
    UnableToAcquireConnection(String),
    UnableToExecuteMigrations(String),
}

impl std::fmt::Display for StorageEngineErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self {
            StorageEngineErrors::UnableToExecuteMigrations(content) => {
                format!("UnableToExecuteMigrations: {}", content)
            }
            StorageEngineErrors::UnableToAcquireConnection(content) => {
                format!("UnableToAcquireConnection: {}", content)
            }
            StorageEngineErrors::InvalidTimestamp(isize) => format!("InvalidTimestamp: {}", isize),
            StorageEngineErrors::NoDataForField(content) => format!("NoDataForField: {}", content),
            StorageEngineErrors::DatabaseError(error) => format!("DatabaseError: {}", error),
        };
        write!(f, "{}", err_msg)
    }
}

#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync + 'static {
    async fn load_storage(config: GlobalConfig) -> Self
    where
        Self: Sized;
    async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>;
    async fn store_user_input_embedding(
        &self,
        user_event_id: uuid::Uuid,
        embedding: &[f32],
    ) -> Result<(), StorageEngineErrors>;
    async fn store_cached_agent_response(
        &self,
        log_event_id: uuid::Uuid,
        user_query_id: uuid::Uuid,
    ) -> Result<(), StorageEngineErrors>;
    async fn get_events<T, F>(
        &self,
        from_timestamp: isize,
        to_timestamp: isize,
        factory_fn: F,
    ) -> Result<Vec<T>, StorageEngineErrors>
    where
        T: Event,
        F: Fn(uuid::Uuid, String, isize, String) -> T + Send,
        Self: Sized;
    async fn get_similar_user_input_embedding(
        &self,
        embedding: Vec<f32>,
    ) -> Result<SlimUserEmbeddingInput, StorageEngineErrors>;
    async fn get_agent_output_for_user_input(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, StorageEngineErrors>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- StorageBackend parsing ---

    #[test]
    fn test_storage_backend_from_str_postgres() {
        let backend: StorageBackend = "postgres".parse().unwrap();
        assert_eq!(backend, StorageBackend::Postgres);
    }

    #[test]
    fn test_storage_backend_from_str_case_insensitive() {
        let backend: StorageBackend = "Postgres".parse().unwrap();
        assert_eq!(backend, StorageBackend::Postgres);
    }

    #[test]
    fn test_storage_backend_from_str_invalid() {
        let result: Result<StorageBackend, _> = "mysql".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_backend_serde_roundtrip() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            backend: StorageBackend,
        }

        let wrapper = Wrapper {
            backend: StorageBackend::Postgres,
        };
        let serialized = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.backend, wrapper.backend);
    }

    // --- UnknownStorageBackendError ---

    #[test]
    fn test_unknown_storage_backend_error_debug() {
        let err = UnknownStorageBackendError;
        assert_eq!(format!("{:?}", err), "UnknownStorageBackendError");
    }

    #[test]
    fn test_unknown_storage_backend_error_display() {
        let err = UnknownStorageBackendError;
        assert_eq!(format!("{}", err), "unknown storage backend");
    }

    // --- StorageEngineErrors display ---

    #[test]
    fn test_storage_engine_errors_display_invalid_timestamp() {
        let err = StorageEngineErrors::InvalidTimestamp(-1);
        assert_eq!(format!("{}", err), "InvalidTimestamp: -1");
    }

    #[test]
    fn test_storage_engine_errors_display_no_data_for_field() {
        let err = StorageEngineErrors::NoDataForField("content".into());
        assert_eq!(format!("{}", err), "NoDataForField: content");
    }

    #[test]
    fn test_storage_engine_errors_display_database_error() {
        let inner =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let err = StorageEngineErrors::DatabaseError(Box::new(inner));
        assert_eq!(format!("{}", err), "DatabaseError: connection refused");
    }

    #[test]
    fn test_storage_engine_errors_display_unable_to_acquire_connection() {
        let err = StorageEngineErrors::UnableToAcquireConnection("timeout".into());
        assert_eq!(format!("{}", err), "UnableToAcquireConnection: timeout");
    }

    #[test]
    fn test_storage_engine_errors_display_unable_to_execute_migrations() {
        let err = StorageEngineErrors::UnableToExecuteMigrations("permission denied".into());
        assert_eq!(
            format!("{}", err),
            "UnableToExecuteMigrations: permission denied"
        );
    }

    // --- StorageBackendProvider ---

    #[test]
    fn test_storage_backend_provider_construct() {
        let provider = StorageBackendProvider {};
        // verify the provider type satisfies Send + Sync
        fn assert_send_sync<T: Send + Sync>(_: &T) {}
        assert_send_sync(&provider);
    }
}
