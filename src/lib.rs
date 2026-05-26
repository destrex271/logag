use std::{iter::Map, sync::OnceLock};

pub mod event_aggregator;
pub mod global_config;
mod shared_log;
mod storage;

// Expose runtime for use across modules.
pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
