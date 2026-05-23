use std::error::Error;

use crate::{global_config::GlobalConfig, shared_log::traits::Event};

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

pub trait StorageEngine {
    async fn load_storage(config: GlobalConfig) -> Self;
    async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>;
    async fn get_events<T, F>(&self, from_timestamp: isize, to_timestamp: isize, factory_fn: F) -> Result<Vec<T>, StorageEngineErrors>
        where
            T: Event,
            F: Fn(uuid::Uuid, String, isize, String) -> T;
}
