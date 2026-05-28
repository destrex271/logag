use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{global_config::GlobalConfig, shared_log::traits::Event, storage::postgres::PostgresStorage};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum StorageBackend{
    Postgres,
}

#[derive(Debug)]
pub struct UnknownStorageBackendError;

impl FromStr for StorageBackend{
    type Err = UnknownStorageBackendError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(StorageBackend::Postgres),
            _ => Err(UnknownStorageBackendError),
        }
    }
}

pub struct StorageBackendProvider{}

impl StorageBackendProvider{
    pub fn get_storage_backend(config: GlobalConfig) -> Box<dyn StorageEngine> {
        match config.storage_backend {
            StorageBackend::Postgres => Box::new(PostgresStorage::new(config)),
        }
    }
}

#[derive(Debug)]
pub enum StorageEngineErrors{
    InvalidTimestamp(isize),
    DatabaseError(Box<dyn Error>),
    NoDataForField(String),
    UnableToAcquireConnection(String),
    UnableToExecuteMigrations(String),
}

impl std::fmt::Display for StorageEngineErrors{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self{
            StorageEngineErrors::UnableToExecuteMigrations(content) => format!("UnableToExecuteMigrations: {}", content),
            StorageEngineErrors::UnableToAcquireConnection(content) => format!("UnableToAcquireConnection: {}", content),
            StorageEngineErrors::InvalidTimestamp(isize) => format!("InvalidTimestamp: {}", isize),
            StorageEngineErrors::NoDataForField(content) => format!("NoDataForField: {}", content),
            StorageEngineErrors::DatabaseError(error) => format!("DatabaseError: {}", error),
        };
        write!(f, "{}", err_msg)
    }
}


#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync + 'static {
    fn new(config: GlobalConfig) -> Self where Self: Sized;
    async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>;
    async fn get_events<T, F>(&self, from_timestamp: isize, to_timestamp: isize, factory_fn: F) -> Result<Vec<T>, StorageEngineErrors>
        where
            T: Event,
            F: Fn(uuid::Uuid, String, isize, String) -> T + Send,
            Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_storage_engine_errors_display_invalid_timestamp() {
        let err = StorageEngineErrors::InvalidTimestamp(-1);
        assert_eq!(format!("{}", err), "InvalidTimestamp: -1");
    }

    #[test]
    fn test_storage_engine_errors_display_no_data_for_field() {
        let err = StorageEngineErrors::NoDataForField("content".into());
        assert_eq!(format!("{}", err), "NoDataForField: content");
    }
}
