use crate::storage::traits::StorageEngineErrors;

pub enum SharedLogErrors {
    UnexpectedStorageLevelError(StorageEngineErrors),
    NoMatchingEntry(String),
    UnableToGenerateEmbeddings(String),
    UnknownError,
}
