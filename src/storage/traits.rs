use std::{error::Error, iter::Map, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{RUNTIME, global_config::GlobalConfig, shared_log::traits::Event, storage::postgres::PostgresStorage};


// Base trait required for any object to be used within a storage
// engine.

pub trait BaseStorageEntity{
    fn construct(raw_data: Map<String, Box<dyn std::any::Any>>) -> Self;
}

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

pub struct StorageEngineFactory{
    storage_engine_refs: Map<StorageBackend, Box<dyn StorageEngine>>,
}

impl StorageEngineFactory{
    fn get_storage_backend(backend: StorageBackend, config: GlobalConfig) -> Result<impl StorageEngine, StorageEngineErrors>{
        match backend{
            StorageBackend::Postgres => {
                let rt = RUNTIME.get().ok_or(StorageEngineErrors::UnableToSpawnBackend("unable to access async runtime for storage engine creation".to_string()))?;

                Ok(
                    rt.block_on(
                        PostgresStorage::load_storage(
                            config.clone()
                        )
                    )
                )

            },
            _ => Err(StorageEngineErrors::UnknownStorageBackend(
                        format!("unknown storage backend {:?}", backend)
                    )
                )
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
    UnableToSpawnBackend(String),
    UnknownStorageBackend(String),
}

impl std::fmt::Display for StorageEngineErrors{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self{
            StorageEngineErrors::UnableToExecuteMigrations(content) => format!("UnableToExecuteMigrations: {}", content),
            StorageEngineErrors::UnableToAcquireConnection(content) => format!("UnableToAcquireConnection: {}", content),
            StorageEngineErrors::InvalidTimestamp(isize) => format!("InvalidTimestamp: {}", isize),
            StorageEngineErrors::NoDataForField(content) => format!("NoDataForField: {}", content),
            StorageEngineErrors::DatabaseError(error) => format!("DatabaseError: {}", error),
            StorageEngineErrors::UnableToSpawnBackend(content) => format!("UnableToSpawnBackend: {}", content),
            StorageEngineErrors::UnknownStorageBackend(content) => format!("UnknownStorageBackend: {}", content),
        };
        write!(f, "{}", err_msg)
    }
}

pub trait StorageEngine<T> where T: Event{
    async fn load_storage(config: GlobalConfig) -> Self where Self:Sized;
    async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>;
    async fn get_events(&self, from_timestamp: isize, to_timestamp: isize, factory_fn: F) -> Result<Vec<T>, StorageEngineErrors>
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
