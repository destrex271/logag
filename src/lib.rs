use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub mod event_aggregator;
pub mod global_config;
mod shared_log;
mod storage;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
