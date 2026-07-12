use crate::storage::traits::StorageEngineErrors;
use std::fmt;

#[derive(Debug)]
pub enum SharedLogErrors {
    UnexpectedStorageLevelError(StorageEngineErrors),
    NoMatchingEntry(String),
    UnableToGenerateEmbeddings(String),
}

impl fmt::Display for SharedLogErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SharedLogErrors::UnexpectedStorageLevelError(err) => {
                write!(f, "unexpected storage level error: {}", err)
            }
            SharedLogErrors::NoMatchingEntry(msg) => {
                write!(f, "no matching entry found: {}", msg)
            }
            SharedLogErrors::UnableToGenerateEmbeddings(msg) => {
                write!(f, "unable to generate embeddings: {}", msg)
            }
        }
    }
}
