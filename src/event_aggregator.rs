use crate::embeddings::fastembed::FastEmbeddingService;
use crate::embeddings::traits::EmbeddingsService;
use crate::global_config::GlobalConfig;
use crate::shared_log::traits::{Event, EventFactory, EventType, SharedLog};
use crate::storage::traits::{StorageBackendProvider, StorageEngine};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use tokio::sync::OnceCell;

pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
}

struct AgentRecorder {
    storage: std::sync::Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: std::sync::Arc<OnceCell<Box<dyn EmbeddingsService>>>,
}

impl AgentRecorder {
    fn new() -> Self {
        AgentRecorder {
            storage: std::sync::Arc::new(OnceCell::new()),
            embedding_model: std::sync::Arc::new(OnceCell::new()),
        }
    }

    fn initialize(&self, config: GlobalConfig) {
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let engine = StorageBackendProvider::get_storage_backend(config).await;
            let _ = storage.set(engine);
        });

        let model = FastEmbeddingService::new().unwrap();
        let _ = self.embedding_model.set(Box::new(model));
    }
}

impl SharedLog for AgentRecorder {
    fn append_event(&self, event: Box<dyn Event>) {
        tracing::info!(
            content = %event.get_content(),
            event_type = %event.get_event_type(),
            id = %event.get_id(),
            "appended event"
        );

        let storage = self.storage.clone();
        tokio::spawn(async move {
            if let Some(engine) = storage.get() {
                let _ = engine.store_event(event.as_ref()).await;
            }
        });
    }
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct MCPEvent {
    event_type: EventType,
    content: String,
    unix_epoch_timestamp: String,
    agent_notes: String,
}

impl Clone for EventAggregator {
    fn clone(&self) -> Self {
        EventAggregator {
            event_factory: EventFactory::new(),
            global_config: self.global_config.clone(),
            shared_log: self.shared_log.clone(),
        }
    }
}

#[tool_router(server_handler)]
impl EventAggregator {
    pub fn new(global_config: GlobalConfig) -> Self {
        let recorder = std::sync::Arc::new(AgentRecorder::new());
        recorder.initialize(global_config.clone());
        EventAggregator {
            event_factory: EventFactory::new(),
            shared_log: recorder,
            global_config,
        }
    }

    #[tool(description = "Register User/Agent/Thinking event")]
    pub fn add_event(
        &self,
        Parameters(MCPEvent {
            event_type,
            content,
            unix_epoch_timestamp,
            agent_notes,
        }): Parameters<MCPEvent>,
    ) -> String {
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                content,
                unix_epoch_timestamp,
                event_type,
            ));
        tracing::info!("{:?}", agent_notes);
        tracing::info!(agent_notes = %agent_notes, "add_event called");
        "success".to_string()
    }

    pub fn add_event_from_raw_stream(
        &self,
        content: String,
        event_type: EventType,
        unix_epoch_timestamp: String,
    ) -> String {
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                content,
                unix_epoch_timestamp,
                event_type,
            ));
        "success".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_log::traits::{Event, EventType};
    use crate::storage::traits::{StorageEngine, StorageEngineErrors};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockStorageEngine {
        store_count: std::sync::Arc<AtomicUsize>,
    }

    #[async_trait]
    impl StorageEngine for MockStorageEngine {
        async fn load_storage(_config: GlobalConfig) -> Self {
            MockStorageEngine {
                store_count: std::sync::Arc::new(AtomicUsize::new(0)),
            }
        }

        async fn store_event(&self, _event: &dyn Event) -> Result<(), StorageEngineErrors> {
            self.store_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn get_events<T, F>(
            &self,
            _from_timestamp: isize,
            _to_timestamp: isize,
            _factory_fn: F,
        ) -> Result<Vec<T>, StorageEngineErrors>
        where
            T: Event,
            F: Fn(uuid::Uuid, String, isize, String) -> T + Send,
            Self: Sized,
        {
            Ok(Vec::new())
        }
    }

    #[cfg(test)]
    impl AgentRecorder {
        fn new_with_storage(engine: Box<dyn StorageEngine>) -> Self {
            let storage = std::sync::Arc::new(OnceCell::new());
            let _ = storage.set(engine);
            let model_cell = std::sync::Arc::new(OnceCell::new());
            let _ = model_cell.set(
                Box::new(FastEmbeddingService::new().unwrap()) as Box<dyn EmbeddingsService>
            );
            AgentRecorder {
                storage,
                embedding_model: model_cell,
            }
        }
    }

    #[test]
    fn test_agent_recorder_new_creates_empty_storage() {
        let recorder = AgentRecorder::new();
        assert!(recorder.storage.get().is_none());
    }

    #[tokio::test]
    async fn test_agent_recorder_append_event_no_storage() {
        let recorder = AgentRecorder::new();
        let event = EventFactory::new().create_log_event(
            "test".into(),
            "1000000".into(),
            EventType::UserInput,
        );
        recorder.append_event(event);
        // Give spawned task time to attempt (and skip) the store
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_agent_recorder_append_event_with_storage() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);

        let event = EventFactory::new().create_log_event(
            "test".into(),
            "1000000".into(),
            EventType::UserInput,
        );
        recorder.append_event(event);

        // Give the spawned task time to execute
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_agent_recorder_append_event_multiple() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);

        for i in 0..5 {
            let event = EventFactory::new().create_log_event(
                format!("event-{}", i),
                format!("{}", 1000000 + i),
                EventType::UserInput,
            );
            recorder.append_event(event);
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_event_aggregator_add_event_with_storage() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);
        let factory = EventFactory::new();
        let global_config = GlobalConfig {
            storage_backend: crate::storage::traits::StorageBackend::Postgres,
            database_connection_string: "postgres://localhost:5432/test".into(),
        };

        let aggregator = EventAggregator {
            event_factory: factory,
            global_config,
            shared_log: std::sync::Arc::new(recorder),
        };

        let mcp_event = MCPEvent {
            event_type: EventType::AgentOutput,
            content: "test output".into(),
            unix_epoch_timestamp: "2000000".into(),
            agent_notes: "notes".into(),
        };

        let result = aggregator.add_event(Parameters(mcp_event));
        assert_eq!(result, "success");

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 1);
    }
}
